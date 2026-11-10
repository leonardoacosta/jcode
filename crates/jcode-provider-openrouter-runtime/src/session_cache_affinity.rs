use super::*;

pub(super) struct SessionCacheAffinity {
    profile: String,
    key: Mutex<String>,
    explicit_header: Option<String>,
}

impl SessionCacheAffinity {
    pub(super) fn new(
        profile: &str,
        headers: &std::collections::BTreeMap<String, String>,
    ) -> Result<Self> {
        let mut explicit_header = None;
        for (name, value) in headers {
            if name.eq_ignore_ascii_case("x-session-id") {
                if explicit_header
                    .as_ref()
                    .is_some_and(|previous| previous != value)
                {
                    anyhow::bail!("session cache affinity: conflicting x-session-id headers");
                }
                explicit_header = Some(value.clone());
            }
        }
        Ok(Self {
            profile: profile.to_owned(),
            key: Mutex::new(new_conversation_id()),
            explicit_header,
        })
    }

    pub(super) fn bind(&self, session: &str) {
        if session.is_empty() {
            return;
        }
        // Domain-separated SHA-256 fingerprints hide raw profile/session identifiers.
        let first = jcode_provider_core::fingerprint::stable_hash_json(&(
            "session-cache-v1",
            &self.profile,
            session,
        ));
        let second = jcode_provider_core::fingerprint::stable_hash_json(&(
            "session-cache-v1-tail",
            &self.profile,
            session,
        ));
        *self.key.lock().unwrap_or_else(|p| p.into_inner()) =
            format!("jcode-{first:016x}{second:016x}");
    }

    pub(super) fn key(&self) -> String {
        self.key.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    pub(super) fn fork(&self) -> Self {
        Self {
            profile: self.profile.clone(),
            key: Mutex::new(new_conversation_id()),
            explicit_header: self.explicit_header.clone(),
        }
    }

    pub(super) fn insert_default(&self, request: &mut Value) {
        request["prompt_cache_key"] =
            Value::String(self.explicit_header.clone().unwrap_or_else(|| self.key()));
    }

    pub(super) fn final_key(&self, request: &Value) -> Result<String> {
        let key = request
            .get("prompt_cache_key")
            .and_then(Value::as_str)
            .filter(|key| !key.trim().is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "session cache affinity: prompt_cache_key must be a nonempty string"
                )
            })?;
        if key.trim_matches([' ', '\t']) != key {
            anyhow::bail!(
                "session cache affinity: prompt_cache_key cannot have boundary spaces or tabs"
            );
        }
        reqwest::header::HeaderValue::from_str(key).map_err(|_| {
            anyhow::anyhow!(
                "session cache affinity: prompt_cache_key is not a valid x-session-id value"
            )
        })?;
        if self
            .explicit_header
            .as_deref()
            .is_some_and(|header| header != key)
        {
            anyhow::bail!(
                "session cache affinity: explicit prompt_cache_key conflicts with x-session-id"
            );
        }
        Ok(key.to_owned())
    }
}
