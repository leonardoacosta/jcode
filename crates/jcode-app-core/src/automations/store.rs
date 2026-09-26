use super::schedule::Schedule;
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

const VERSION: u32 = 1;
const MAX_RUNS: usize = 1000;
const MAX_OUTPUT: usize = 64 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Automation {
    pub id: String,
    pub skill: String,
    pub arguments: String,
    pub working_dir: PathBuf,
    pub schedule: Schedule,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub next_due: DateTime<Utc>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Run {
    pub id: String,
    pub automation_id: String,
    pub session_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: RunStatus,
    pub output: String,
    pub truncated: bool,
    pub skill_source: Option<String>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Running,
    Success,
    Failed,
    Blocked,
    Timeout,
    Interrupted,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct State {
    version: u32,
    automations: Vec<Automation>,
    runs: Vec<Run>,
}
pub struct Store {
    path: PathBuf,
    lock: File,
    state: State,
    poisoned: bool,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path.with_extension("lock"))?;
        lock.try_lock()
            .context("automation store already owned by another daemon")?;
        let state = if path.exists() {
            let value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)
                .context("corrupt automation state preserved")?;
            let version = value
                .get("version")
                .and_then(|v| v.as_u64())
                .context("automation state missing version")?;
            if version != u64::from(VERSION) {
                bail!("unsupported automation state version {version}")
            }
            serde_json::from_value(value).context("invalid automation state preserved")?
        } else {
            State {
                version: VERSION,
                automations: Vec::new(),
                runs: Vec::new(),
            }
        };
        Ok(Self {
            path,
            lock,
            state,
            poisoned: false,
        })
    }
    pub fn automations(&self) -> &[Automation] {
        &self.state.automations
    }
    pub fn runs(&self) -> &[Run] {
        &self.state.runs
    }
    fn commit(&mut self, next: State) -> Result<()> {
        if self.poisoned {
            bail!("automation store persistence failed; reopen after restart")
        }
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let temp = self.path.with_extension(format!(
            "tmp-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let result = (|| -> Result<()> {
            let mut opts = OpenOptions::new();
            opts.create_new(true).write(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }
            let mut file = opts.open(&temp)?;
            serde_json::to_writer(&mut file, &next)?;
            file.sync_all()?;
            fs::rename(&temp, &self.path)?;
            File::open(parent)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temp);
        }
        if let Err(error) = result {
            self.poisoned = true;
            return Err(error);
        }
        self.state = next;
        Ok(())
    }
    fn validate_skill(skill: &str, dir: &Path) -> Result<()> {
        if !dir.is_absolute() || !dir.is_dir() {
            bail!("working_dir must be an existing absolute directory")
        }
        let registry = jcode_base::skill::SkillRegistry::load_for_working_dir(Some(dir))
            .context("load skills")?;
        if !registry.contains(skill) {
            bail!("unknown skill: {skill}")
        }
        Ok(())
    }
    pub fn create(&mut self, mut automation: Automation, _skills_dir: &Path) -> Result<()> {
        automation.schedule.validate()?;
        Self::validate_skill(&automation.skill, &automation.working_dir)?;
        if self.state.automations.iter().any(|a| a.id == automation.id) {
            bail!("duplicate automation ID")
        }
        automation.next_due = automation.schedule.next_after(automation.created_at)?;
        let mut next = self.state.clone();
        next.automations.push(automation);
        self.commit(next)
    }
    pub fn update(
        &mut self,
        mut automation: Automation,
        _skills_dir: &Path,
        now: DateTime<Utc>,
    ) -> Result<()> {
        automation.schedule.validate()?;
        Self::validate_skill(&automation.skill, &automation.working_dir)?;
        let i = self
            .state
            .automations
            .iter()
            .position(|a| a.id == automation.id)
            .context("automation not found")?;
        automation.next_due = automation.schedule.next_after(now)?;
        let mut next = self.state.clone();
        next.automations[i] = automation;
        self.commit(next)
    }
    pub fn pause(&mut self, id: &str) -> Result<()> {
        let mut next = self.state.clone();
        next.automations
            .iter_mut()
            .find(|a| a.id == id)
            .context("automation not found")?
            .enabled = false;
        self.commit(next)
    }
    pub fn resume(&mut self, id: &str, now: DateTime<Utc>) -> Result<()> {
        let mut next = self.state.clone();
        let a = next
            .automations
            .iter_mut()
            .find(|a| a.id == id)
            .context("automation not found")?;
        a.next_due = a.schedule.next_after(now)?;
        a.enabled = true;
        a.error = None;
        self.commit(next)
    }
    pub fn claim_due(&mut self, now: DateTime<Utc>) -> Result<Option<(Automation, Run)>> {
        if self.poisoned {
            bail!("automation store persistence failed; reopen after restart")
        }
        if self
            .state
            .runs
            .iter()
            .any(|r| r.status == RunStatus::Running)
        {
            return Ok(None);
        }
        let Some(i) = self
            .state
            .automations
            .iter()
            .enumerate()
            .filter(|(_, a)| a.enabled && a.next_due <= now)
            .min_by_key(|(_, a)| (a.next_due, &a.id))
            .map(|(i, _)| i)
        else {
            return Ok(None);
        };
        let mut next = self.state.clone();
        let a = &mut next.automations[i];
        let claimed = a.clone();
        a.next_due = a.schedule.next_from(a.next_due, now)?;
        let run = Run {
            id: uuid::Uuid::new_v4().to_string(),
            automation_id: a.id.clone(),
            session_id: None,
            started_at: now,
            ended_at: None,
            status: RunStatus::Running,
            output: String::new(),
            truncated: false,
            skill_source: None,
        };
        next.runs.push(run.clone());
        self.commit(next)?;
        Ok(Some((claimed, run)))
    }
    pub fn finish(
        &mut self,
        id: &str,
        status: RunStatus,
        output: &str,
        now: DateTime<Utc>,
    ) -> Result<()> {
        if status == RunStatus::Running {
            bail!("cannot finish as running")
        }
        let mut next = self.state.clone();
        let active = next
            .runs
            .iter()
            .find(|r| r.id == id && r.status == RunStatus::Running)
            .context("active run not found")?;
        let started_at = active.started_at;
        let automation_id = active.automation_id.clone();
        let run = next.runs.iter_mut().find(|r| r.id == id).unwrap();
        let mut end = output.len().min(MAX_OUTPUT);
        while !output.is_char_boundary(end) {
            end -= 1
        }
        run.output = output[..end].to_string();
        run.truncated = end < output.len();
        run.status = status;
        run.ended_at = Some(now);
        let mut old: Vec<_> = next
            .runs
            .iter()
            .filter(|r| r.status != RunStatus::Running)
            .map(|r| (r.ended_at.unwrap_or(r.started_at), r.id.clone()))
            .collect();
        old.sort_by_key(|x| x.0);
        let excess = old.len().saturating_sub(MAX_RUNS);
        let remove: std::collections::HashSet<_> =
            old.into_iter().take(excess).map(|x| x.1).collect();
        next.runs.retain(|r| !remove.contains(&r.id));
        if let Some(a) = next.automations.iter_mut().find(|a| a.id == automation_id) {
            if a.next_due <= now {
                if let Ok(due) = a.schedule.next_from(a.next_due, now.max(started_at)) {
                    a.next_due = due;
                } else {
                    a.enabled = false;
                    a.error = Some("schedule could not be resolved after run".into());
                }
            }
        }
        self.commit(next)
    }
    pub fn recover(&mut self, now: DateTime<Utc>) -> Result<()> {
        let mut next = self.state.clone();
        for r in &mut next.runs {
            if r.status == RunStatus::Running {
                r.status = RunStatus::Interrupted;
                r.ended_at = Some(now)
            }
        }
        for a in &mut next.automations {
            if a.enabled {
                if let Err(error) = a.schedule.validate() {
                    a.enabled = false;
                    a.error = Some(error.to_string());
                    continue;
                }
            }
            if a.enabled && a.next_due <= now {
                match a.schedule.next_from(a.next_due, now) {
                    Ok(due) => a.next_due = due,
                    Err(error) => {
                        a.enabled = false;
                        a.error = Some(error.to_string())
                    }
                }
            }
        }
        self.commit(next)
    }
    pub fn set_session_and_source(
        &mut self,
        id: &str,
        session_id: Option<String>,
        skill_source: Option<String>,
    ) -> Result<()> {
        let mut next = self.state.clone();
        let r = next
            .runs
            .iter_mut()
            .find(|r| r.id == id && r.status == RunStatus::Running)
            .context("active run not found")?;
        r.session_id = session_id;
        r.skill_source = skill_source;
        self.commit(next)
    }
}
impl Drop for Store {
    fn drop(&mut self) {
        let _ = self.lock.unlock();
    }
}
