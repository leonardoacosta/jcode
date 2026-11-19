use super::*;
use std::ffi::OsString;

struct Home {
    _guard: std::sync::MutexGuard<'static, ()>,
    dir: tempfile::TempDir,
    prev_home: Option<OsString>,
    prev_cwd: PathBuf,
}

impl Home {
    fn new() -> Self {
        let guard = crate::storage::lock_test_env();
        let prev_home = std::env::var_os("JCODE_HOME");
        let prev_cwd = std::env::current_dir().expect("cwd");
        let dir = tempfile::TempDir::new().expect("tempdir");
        crate::env::set_var("JCODE_HOME", dir.path());
        Config::invalidate_cache();
        Self {
            _guard: guard,
            dir,
            prev_home,
            prev_cwd,
        }
    }

    fn write(&self, rel: &str, text: &str) {
        let path = self.dir.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    fn root(&self) -> PathBuf {
        self.dir.path().join("config.toml")
    }

    fn load(&self) -> anyhow::Result<Config> {
        Config::load_strict()
    }

    fn err(&self) -> String {
        format!("{:#}", self.load().expect_err("expected load failure"))
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.prev_cwd);
        match self.prev_home.take() {
            Some(prev) => crate::env::set_var("JCODE_HOME", prev),
            None => crate::env::remove_var("JCODE_HOME"),
        }
        Config::invalidate_cache();
    }
}

#[test]
fn absent_manifest_keeps_single_file_behavior_and_unknown_keys() {
    let home = Home::new();
    home.write(
        "config.toml",
        "future_table = { x = 1 }\n[display]\ncentered = true\n",
    );
    let config = home.load().unwrap();
    assert!(config.display.centered);
    let table = Config::read_composed_table(&home.root()).unwrap();
    assert!(table.contains_key("future_table"));
}

#[test]
fn explicit_order_root_wins_and_siblings_survive() {
    let home = Home::new();
    home.write(
        "config.toml",
        "include = [\"config/a.toml\", \"config/b.toml\"]\n[display]\ncentered = true\n",
    );
    home.write(
        "config/a.toml",
        "[display]\ncentered = false\npin_todos = true\ncompact_notifications = true\n[agents]\nswarm_model = \"a-model\"\n",
    );
    home.write(
        "config/b.toml",
        "[display]\ncompact_notifications = false\n[agents]\nswarm_model = \"b-model\"\n",
    );
    let config = home.load().unwrap();
    assert!(config.display.centered, "root beats fragments");
    assert!(
        config.display.pin_todos,
        "sibling from first fragment survives"
    );
    assert!(
        !config.display.compact_notifications,
        "last fragment beats earlier fragment"
    );
    assert_eq!(config.agents.swarm_model.as_deref(), Some("b-model"));
}

#[test]
fn composition_is_independent_of_working_directory() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/ui.toml\"]\n");
    home.write("config/ui.toml", "[display]\ncentered = true\n");
    // A decoy relative to cwd must never be read.
    let elsewhere = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(elsewhere.path().join("config")).unwrap();
    std::fs::write(
        elsewhere.path().join("config/ui.toml"),
        "[display]\ncentered = false\n",
    )
    .unwrap();
    std::env::set_current_dir(elsewhere.path()).unwrap();
    assert!(home.load().unwrap().display.centered);
}

#[test]
fn directory_is_never_scanned() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/ui.toml\"]\n");
    home.write("config/ui.toml", "[display]\ncentered = true\n");
    home.write("config/extra.toml", "[agents]\nswarm_model = \"extra\"\n");
    home.write("config.d/zz.toml", "[agents]\nswarm_model = \"zz\"\n");
    assert!(home.load().unwrap().agents.swarm_model.is_none());
}

fn provider_fragment(models: &str) -> String {
    format!("[providers.example]\nbase_url = \"https://example.test/v1\"\nmodels = {models}\n")
}

#[test]
fn model_arrays_replace_including_empty_arrays() {
    let home = Home::new();
    home.write(
        "config.toml",
        "include = [\"config/p1.toml\", \"config/p2.toml\"]\n",
    );
    home.write(
        "config/p1.toml",
        &provider_fragment("[{ id = \"one\" }, { id = \"two\" }]"),
    );
    home.write("config/p2.toml", &provider_fragment("[{ id = \"three\" }]"));
    let config = home.load().unwrap();
    let models = &config.providers["example"].models;
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "three");

    home.write("config/p2.toml", &provider_fragment("[]"));
    let config = home.load().unwrap();
    assert!(config.providers["example"].models.is_empty());
    assert_eq!(
        config.providers["example"].base_url,
        "https://example.test/v1"
    );
}

#[test]
fn array_of_tables_replace() {
    let home = Home::new();
    home.write(
        "config.toml",
        "include = [\"config/a.toml\"]\n[[providers.example.models]]\nid = \"root\"\n",
    );
    home.write(
        "config/a.toml",
        "[[providers.example.models]]\nid = \"x\"\n[[providers.example.models]]\nid = \"y\"\n",
    );
    let config = home.load().unwrap();
    let models = &config.providers["example"].models;
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "root");
}

#[test]
fn final_type_validation_runs_after_merge() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/a.toml\"]\n");
    home.write(
        "config/a.toml",
        "[display]\ncentered = \"sentinel-secret-value\"\n",
    );
    let error = home.err();
    assert!(error.contains("config.toml"), "{error}");
    assert!(!error.contains("sentinel-secret-value"), "{error}");
}

#[test]
fn shape_conflicts_name_key_and_both_sources() {
    let home = Home::new();
    home.write(
        "config.toml",
        "include = [\"config/a.toml\", \"config/b.toml\"]\n",
    );
    home.write("config/a.toml", "[display]\ncentered = true\n");
    home.write("config/b.toml", "display = \"sentinel-secret-value\"\n");
    let error = home.err();
    assert!(error.contains("`display`"), "{error}");
    assert!(
        error.contains("a.toml") && error.contains("b.toml"),
        "{error}"
    );
    assert!(!error.contains("sentinel-secret-value"), "{error}");

    // Scalar first, table later, and conflict against the root itself.
    home.write("config/a.toml", "display = 1\n");
    home.write("config/b.toml", "[agents]\nswarm_model = \"m\"\n");
    home.write(
        "config.toml",
        "include = [\"config/a.toml\", \"config/b.toml\"]\n[display]\ncentered = true\n",
    );
    let error = home.err();
    assert!(
        error.contains("a.toml") && error.contains("config.toml"),
        "{error}"
    );
}

#[test]
fn nested_key_conflicts_name_both_sources() {
    let home = Home::new();
    home.write(
        "config.toml",
        "include = [\"config/a.toml\", \"config/b.toml\"]\n",
    );
    // `a.toml` introduces the whole `[display.nest]` table at once.
    home.write("config/a.toml", "[display.nest]\ninner = true\n");
    home.write("config/b.toml", "[display]\nnest = { inner = { x = 1 } }\n");
    let error = home.err();
    assert!(error.contains("display.nest.inner"), "{error}");
    assert!(
        error.contains("a.toml") && error.contains("b.toml"),
        "both sources named, no vague fallback: {error}"
    );
    assert!(!error.contains("an earlier source"), "{error}");

    // Scalar arriving over a table inserted whole by an earlier fragment.
    home.write(
        "config/b.toml",
        "[display]\nnest = \"sentinel-secret-value\"\n",
    );
    let error = home.err();
    assert!(error.contains("display.nest"), "{error}");
    assert!(
        error.contains("a.toml") && error.contains("b.toml"),
        "{error}"
    );
    assert!(!error.contains("sentinel-secret-value"), "{error}");
}

#[test]
fn error_text_redacts_literal_numeric_and_multiline_values() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/a.toml\"]\n");
    let cases = [
        // literal string where a bool is expected
        "[display]\ncentered = 'sentinel-literal-1'\n",
        // multi-line basic and literal strings
        "[display]\ncentered = \"\"\"\nsentinel-multi-2\nsecond-line\n\"\"\"\n",
        "[display]\ncentered = '''\nsentinel-multi-3\nthird-line\n'''\n",
        // numbers and floats where a bool is expected
        "[display]\ncentered = 98765432\n",
        "[display]\ncentered = 3.14159\n",
        // wrong value for an enum
        "[agents]\nswarm_spawn_mode = \"sentinel-enum-4\"\n",
        // secret-looking key with a wrong-typed value
        "[display]\ncentered = { api_key = \"sentinel-inline-5\" }\n",
    ];
    for body in cases {
        home.write("config/a.toml", body);
        let error = home.err();
        for leak in [
            "sentinel-",
            "second-line",
            "third-line",
            "98765432",
            "3.14159",
        ] {
            assert!(
                !error.contains(leak),
                "leaked `{leak}` for {body:?}: {error}"
            );
        }
        assert!(error.contains("config.toml"), "{error}");
    }
    // Syntax errors on a literal/number tail do not echo it either.
    home.write(
        "config/a.toml",
        "[display]\ncentered = 'sentinel-syn-6' junk 424242\n",
    );
    let error = home.err();
    assert!(
        !error.contains("sentinel-") && !error.contains("424242"),
        "{error}"
    );
}

#[test]
fn unknown_keys_in_fragments_are_accepted() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/a.toml\"]\n");
    home.write("config/a.toml", "[future]\nknob = 1\n[tasks]\n");
    home.load().unwrap();
    let table = Config::read_composed_table(&home.root()).unwrap();
    assert!(table.contains_key("future") && !table.contains_key("include"));
}

#[test]
fn missing_and_directory_includes_fail_with_source() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/missing.toml\"]\n");
    let error = home.err();
    assert!(
        error.contains("missing.toml") && error.contains("does not exist"),
        "{error}"
    );

    std::fs::create_dir_all(home.dir.path().join("config/dir.toml")).unwrap();
    home.write("config.toml", "include = [\"config/dir.toml\"]\n");
    let error = home.err();
    assert!(
        error.contains("dir.toml") && error.contains("regular file"),
        "{error}"
    );
}

#[test]
fn malformed_fragment_reports_path_and_line_without_excerpt() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/bad.toml\"]\n");
    home.write(
        "config/bad.toml",
        "[display]\ncentered = true\napi_key = \"sentinel-secret-value\" trailing\n",
    );
    let error = home.err();
    assert!(error.contains("bad.toml:3:"), "{error}");
    assert!(!error.contains("sentinel-secret-value"), "{error}");
}

#[test]
fn unreadable_fragment_is_an_error() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/bin.toml\"]\n");
    std::fs::write(
        home.dir.path().join("config.toml"),
        "include = [\"config/bin.toml\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(home.dir.path().join("config")).unwrap();
    std::fs::write(home.dir.path().join("config/bin.toml"), [0xff, 0xfe, 0x00]).unwrap();
    let error = home.err();
    assert!(
        error.contains("bin.toml") && error.contains("UTF-8"),
        "{error}"
    );
}

#[test]
fn nested_self_and_duplicate_includes_are_rejected() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/a.toml\"]\n");
    home.write("config/a.toml", "include = [\"b.toml\"]\n");
    let error = home.err();
    assert!(
        error.contains("nested") && error.contains("a.toml"),
        "{error}"
    );

    home.write("config.toml", "include = [\"config.toml\"]\n");
    let error = home.err();
    assert!(error.contains("root file itself"), "{error}");

    home.write("config/a.toml", "[display]\ncentered = true\n");
    home.write(
        "config.toml",
        "include = [\"config/a.toml\", \"./config/../config/a.toml\"]\n",
    );
    // `..` is rejected before duplicate detection, so use a normalized duplicate.
    home.write(
        "config.toml",
        "include = [\"config/a.toml\", \"./config/a.toml\"]\n",
    );
    let error = home.err();
    assert!(error.contains("duplicates"), "{error}");
}

#[test]
fn unsafe_include_syntax_is_rejected() {
    let home = Home::new();
    home.write("config/a.toml", "[display]\ncentered = true\n");
    for entry in [
        "../outside.toml",
        "config/../config/a.toml",
        "/etc/passwd.toml",
        "~/a.toml",
        "$HOME/a.toml",
        "config/*.toml",
        "config/a.txt",
        "",
    ] {
        home.write("config.toml", &format!("include = [{entry:?}]\n"));
        assert!(home.load().is_err(), "{entry:?} must be rejected");
    }
    home.write("config.toml", "include = \"config/a.toml\"\n");
    assert!(home.err().contains("array"));
    home.write("config.toml", "include = [1]\n");
    assert!(home.err().contains("strings"));
}

#[cfg(unix)]
#[test]
fn symlink_escape_rejected_and_contained_symlink_allowed() {
    let home = Home::new();
    let outside = tempfile::TempDir::new().unwrap();
    std::fs::write(
        outside.path().join("o.toml"),
        "[display]\ncentered = true\n",
    )
    .unwrap();
    std::fs::create_dir_all(home.dir.path().join("config")).unwrap();
    std::os::unix::fs::symlink(
        outside.path().join("o.toml"),
        home.dir.path().join("config/escape.toml"),
    )
    .unwrap();
    home.write("config.toml", "include = [\"config/escape.toml\"]\n");
    let error = home.err();
    assert!(error.contains("outside the config directory"), "{error}");

    home.write("real/ui.toml", "[display]\ncentered = true\n");
    std::os::unix::fs::symlink(
        home.dir.path().join("real/ui.toml"),
        home.dir.path().join("config/link.toml"),
    )
    .unwrap();
    home.write("config.toml", "include = [\"config/link.toml\"]\n");
    assert!(home.load().unwrap().display.centered);
}

#[test]
fn environment_overrides_apply_after_composition() {
    let home = Home::new();
    let prev = std::env::var_os("JCODE_UPDATE_CHANNEL");
    home.write("config.toml", "include = [\"config/a.toml\"]\n");
    home.write("config/a.toml", "[features]\nupdate_channel = \"stable\"\n");
    crate::env::set_var("JCODE_UPDATE_CHANNEL", "main");
    let strict = Config::load_strict().unwrap();
    let file_only = Config::load_from_file_strict().unwrap().unwrap();
    match prev {
        Some(prev) => crate::env::set_var("JCODE_UPDATE_CHANNEL", prev),
        None => crate::env::remove_var("JCODE_UPDATE_CHANNEL"),
    }
    assert_eq!(strict.features.update_channel, UpdateChannel::Main);
    assert_eq!(file_only.features.update_channel, UpdateChannel::Stable);
}

#[test]
fn source_fingerprint_tracks_membership_content_and_missing_states() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/a.toml\"]\n");
    home.write("config/a.toml", "[display]\ncentered = true\n");
    let read = || super::composition::SourceFingerprint::read(Some(home.root()));
    let base = read();
    assert_eq!(base, read());

    home.write("config/a.toml", "[display]\ncentered = false\n");
    assert_ne!(base, read(), "equal-length fragment edit");

    home.write("config/a.toml", "[display]\ncentered = true\n");
    assert_eq!(base, read(), "restoring content restores identity");

    std::fs::remove_file(home.dir.path().join("config/a.toml")).unwrap();
    let missing = read();
    assert_ne!(base, missing, "deletion");
    home.write("config/a.toml", "[display]\ncentered = true\n");
    assert_eq!(base, read(), "restoration");

    home.write(
        "config.toml",
        "include = [\"config/a.toml\", \"config/b.toml\"]\n",
    );
    assert_ne!(base, read(), "manifest membership");
}

fn read(home: &Home, rel: &str) -> String {
    std::fs::read_to_string(home.dir.path().join(rel)).unwrap()
}

fn split_fixture(home: &Home) {
    home.write(
        "config.toml",
        "# my root\ninclude = [\"config/ui.toml\", \"config/providers/example.toml\"]\nfuture_table = { keep = 1 }\n\n[display]\n# keep this comment\npin_todos = true\n",
    );
    home.write(
        "config/ui.toml",
        "[display]\ncentered = true\ncompact_notifications = true\n",
    );
    home.write(
        "config/providers/example.toml",
        &provider_fragment("[{ id = \"one\" }, { id = \"two\" }]"),
    );
}

#[test]
fn setter_on_fragment_value_writes_only_a_root_override() {
    let home = Home::new();
    split_fixture(&home);
    let ui_before = read(&home, "config/ui.toml");
    let prov_before = read(&home, "config/providers/example.toml");
    Config::set_display_centered(false).unwrap();

    assert_eq!(
        read(&home, "config/ui.toml"),
        ui_before,
        "fragment untouched"
    );
    assert_eq!(read(&home, "config/providers/example.toml"), prov_before);
    let root = read(&home, "config.toml");
    assert!(
        root.contains("# my root") && root.contains("# keep this comment"),
        "{root}"
    );
    assert!(root.contains("include = [\"config/ui.toml\""), "{root}");
    assert!(root.contains("future_table"), "{root}");
    assert!(root.contains("centered = false"), "{root}");
    assert!(
        !root.contains("[providers"),
        "sparse: no catalog baked in: {root}"
    );
    assert!(!root.contains("compact_notifications"), "{root}");
    let config = home.load().unwrap();
    assert!(!config.display.centered);
    assert!(config.display.compact_notifications);
    assert_eq!(config.providers["example"].models.len(), 2);
}

const LEGACY_REALISTIC: &str = "# legacy jcode config\r\nfuture_inline = { knob = 7, name = 'lit' } # unknown inline table\r\ndisplay.pin_todos = true # dotted key keeps this comment\r\ngateway = { port = 7777 }\r\n\r\n[future]\r\nwin_path = 'C:\\Users\\me\\jcode' # literal string\r\nlist = [ 1, 2,\r\n  3 ]\r\n\r\n[keybindings]\r\nscroll_up   =   \"ctrl+y\"   # odd spacing\r\n";

#[test]
fn realistic_legacy_file_round_trip_changes_only_the_requested_leaf() {
    let home = Home::new();
    home.write("config.toml", LEGACY_REALISTIC);
    Config::set_pin_todos(false).unwrap();
    let after = read(&home, "config.toml");
    assert_eq!(
        after,
        LEGACY_REALISTIC.replace("pin_todos = true", "pin_todos = false"),
        "only the requested leaf changes: CRLF, comments, spacing, literal \
         strings, dotted keys and inline tables are byte-identical"
    );
    assert!(!home.load().unwrap().display.pin_todos);

    // Setting another leaf in an inline table also leaves the rest alone.
    Config::set_pin_todos(true).unwrap();
    assert_eq!(read(&home, "config.toml"), LEGACY_REALISTIC);
}

#[test]
fn legacy_file_without_include_is_untouched_on_non_change_paths() {
    let home = Home::new();
    home.write("config.toml", LEGACY_REALISTIC);
    let bytes = std::fs::read(home.root()).unwrap();
    let modified = std::fs::metadata(home.root()).unwrap().modified().unwrap();

    // Composition of a file with no manifest is exactly the plain TOML parse.
    let composed = Config::read_composed_table(&home.root()).unwrap();
    assert_eq!(composed, LEGACY_REALISTIC.parse::<toml::Table>().unwrap());
    home.load().unwrap();
    Config::load_for_update().unwrap();
    Config::load_for_update().unwrap().save().unwrap();
    let _ = crate::config::config();
    let _ = crate::config::config_strict();
    assert_eq!(
        std::fs::read(home.root()).unwrap(),
        bytes,
        "no bytes change"
    );
    assert_eq!(
        std::fs::metadata(home.root()).unwrap().modified().unwrap(),
        modified,
        "no write happened"
    );
    assert!(
        !home.dir.path().join("config").exists(),
        "no fragment directory is created"
    );
}

#[cfg(unix)]
#[test]
fn new_root_file_uses_default_permissions_like_the_previous_writer() {
    use std::os::unix::fs::PermissionsExt;
    let home = Home::new();
    assert!(!home.root().exists());
    Config::set_display_centered(true).unwrap();
    // `OpenOptions::create_new` and the old `fs::write` both honour the umask,
    // so a new root is not forced to 0600 here. This documents that behavior.
    let mode = std::fs::metadata(home.root()).unwrap().permissions().mode() & 0o777;
    let umask_default = {
        let probe = home.dir.path().join("umask-probe");
        std::fs::write(&probe, "").unwrap();
        std::fs::metadata(&probe).unwrap().permissions().mode() & 0o777
    };
    assert_eq!(mode, umask_default);
}

#[test]
fn legacy_single_file_save_is_sparse_and_keeps_comments_and_unknown_tables() {
    let home = Home::new();
    home.write(
        "config.toml",
        "# top comment\n[display]\ncentered = true # inline\n[tasks]\n[worktrees]\n[future]\nknob = 7\n",
    );
    Config::set_pin_todos(false).unwrap();
    let root = read(&home, "config.toml");
    assert!(
        root.contains("# top comment") && root.contains("centered = true # inline"),
        "{root}"
    );
    assert!(
        root.contains("[future]") && root.contains("knob = 7"),
        "{root}"
    );
    assert!(
        root.contains("[tasks]") && root.contains("[worktrees]"),
        "{root}"
    );
    assert!(root.contains("pin_todos = false"), "{root}");
    assert!(
        !root.contains("[keybindings]") && !root.contains("[provider]"),
        "{root}"
    );
    assert!(!home.load().unwrap().display.pin_todos);
}

#[test]
fn no_op_save_does_not_write_and_new_file_is_sparse() {
    let home = Home::new();
    split_fixture(&home);
    let before = std::fs::metadata(home.root()).unwrap().modified().unwrap();
    let text = read(&home, "config.toml");
    Config::load_for_update().unwrap().save().unwrap();
    assert_eq!(read(&home, "config.toml"), text);
    assert_eq!(
        std::fs::metadata(home.root()).unwrap().modified().unwrap(),
        before
    );

    drop(home);
    let fresh = Home::new();
    assert!(!fresh.root().exists());
    Config::set_display_centered(true).unwrap();
    let written = read(&fresh, "config.toml");
    assert_eq!(written.trim(), "[display]\ncentered = true", "{written}");
}

#[test]
fn environment_overrides_are_not_persisted_by_setters() {
    let home = Home::new();
    let prev = std::env::var_os("JCODE_UPDATE_CHANNEL");
    split_fixture(&home);
    crate::env::set_var("JCODE_UPDATE_CHANNEL", "main");
    let result = Config::set_pin_todos(false);
    match prev {
        Some(prev) => crate::env::set_var("JCODE_UPDATE_CHANNEL", prev),
        None => crate::env::remove_var("JCODE_UPDATE_CHANNEL"),
    }
    result.unwrap();
    let root = read(&home, "config.toml");
    assert!(!root.contains("update_channel"), "{root}");
}

#[test]
fn provider_model_array_override_replaces_whole_array_in_root() {
    let home = Home::new();
    split_fixture(&home);
    let mut config = Config::load_for_update().unwrap();
    config
        .providers
        .get_mut("example")
        .unwrap()
        .models
        .truncate(1);
    config.save().unwrap();
    assert_eq!(
        read(&home, "config/providers/example.toml"),
        provider_fragment("[{ id = \"one\" }, { id = \"two\" }]")
    );
    let loaded = home.load().unwrap();
    assert_eq!(loaded.providers["example"].models.len(), 1);

    // An emptied array stays representable as an explicit empty override.
    let mut config = Config::load_for_update().unwrap();
    config.providers.get_mut("example").unwrap().models.clear();
    config.save().unwrap();
    assert!(home.load().unwrap().providers["example"].models.is_empty());
}

#[test]
fn unrepresentable_removal_of_inherited_value_fails_without_mutation() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/a.toml\"]\n");
    home.write(
        "config/a.toml",
        "[provider]\ndefault_model = \"inherited-model\"\n",
    );
    let root_before = read(&home, "config.toml");
    let error = format!(
        "{:#}",
        Config::set_default_model_only(None).expect_err("must refuse")
    );
    assert!(error.contains("provider.default_model"), "{error}");
    assert!(error.contains("a.toml"), "{error}");
    assert_eq!(read(&home, "config.toml"), root_before);
}

#[test]
fn removing_a_root_override_that_masks_nothing_works() {
    let home = Home::new();
    home.write("config.toml", "[provider]\ndefault_model = \"m\"\n");
    Config::set_default_model_only(None).unwrap();
    let root = read(&home, "config.toml");
    assert!(!root.contains("default_model"), "{root}");
}

#[test]
fn save_refuses_invalid_composition_without_touching_files() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/missing.toml\"]\n");
    let before = read(&home, "config.toml");
    assert!(Config::set_pin_todos(true).is_err());
    assert_eq!(read(&home, "config.toml"), before);
    let leftovers: Vec<_> = std::fs::read_dir(home.dir.path())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("jcode-tmp"))
        .collect();
    assert!(leftovers.is_empty());
}

fn temp_leftovers(home: &Home) -> usize {
    std::fs::read_dir(home.dir.path())
        .unwrap()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().contains("jcode-tmp"))
        .count()
}

#[cfg(unix)]
#[test]
fn replacement_keeps_permissions_and_symlink() {
    use std::os::unix::fs::PermissionsExt;
    let home = Home::new();
    home.write("real/config.toml", "[display]\ncentered = false\n");
    std::fs::set_permissions(
        home.dir.path().join("real/config.toml"),
        std::fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    std::os::unix::fs::symlink(home.dir.path().join("real/config.toml"), home.root()).unwrap();
    Config::set_display_centered(true).unwrap();
    assert!(
        std::fs::symlink_metadata(home.root())
            .unwrap()
            .file_type()
            .is_symlink()
    );
    let target = home.dir.path().join("real/config.toml");
    assert_eq!(
        std::fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(
        std::fs::read_to_string(&target)
            .unwrap()
            .contains("centered = true")
    );
    assert_eq!(temp_leftovers(&home), 0);
}

use super::sparse_save::{TEST_FAULT, TestFault};

fn set_fault(fault: TestFault) {
    *TEST_FAULT.lock().unwrap() = Some(fault);
}

#[test]
fn concurrent_fragment_edit_before_replace_is_refused() {
    let home = Home::new();
    split_fixture(&home);
    let root_before = read(&home, "config.toml");
    let fragment = home.dir.path().join("config/ui.toml");
    set_fault(TestFault::BeforeReplace(Box::new(move || {
        std::fs::write(
            fragment,
            "[display]\ncentered = false\ncompact_notifications = true\n",
        )
        .unwrap();
    })));
    let error = format!(
        "{:#}",
        Config::set_pin_todos(false).expect_err("drift must refuse")
    );
    assert!(error.contains("changed while saving"), "{error}");
    assert_eq!(read(&home, "config.toml"), root_before);
    assert_eq!(temp_leftovers(&home), 0);
}

#[test]
fn concurrent_root_edit_before_replace_is_refused() {
    let home = Home::new();
    split_fixture(&home);
    let root = home.root();
    set_fault(TestFault::BeforeReplace(Box::new({
        let root = root.clone();
        move || std::fs::write(root, "[display]\ncentered = false\n").unwrap()
    })));
    assert!(Config::set_pin_todos(false).is_err());
    assert_eq!(read(&home, "config.toml"), "[display]\ncentered = false\n");
    assert_eq!(temp_leftovers(&home), 0);
}

#[cfg(unix)]
#[test]
fn write_and_replace_failures_preserve_original_bytes_and_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let home = Home::new();
    split_fixture(&home);
    std::fs::set_permissions(home.root(), std::fs::Permissions::from_mode(0o640)).unwrap();
    let before = read(&home, "config.toml");
    for fault in [TestFault::FailWrite, TestFault::FailReplace] {
        set_fault(fault);
        assert!(Config::set_pin_todos(false).is_err());
        assert_eq!(read(&home, "config.toml"), before);
        assert_eq!(
            std::fs::metadata(home.root()).unwrap().permissions().mode() & 0o777,
            0o640
        );
        assert_eq!(temp_leftovers(&home), 0);
    }
}

#[test]
fn invalid_baseline_blocks_save_and_keeps_bytes() {
    let home = Home::new();
    home.write("config.toml", "[display\ncentered = true\n");
    let before = read(&home, "config.toml");
    assert!(Config::set_pin_todos(true).is_err());
    assert_eq!(read(&home, "config.toml"), before);
}

#[test]
fn legacy_migrations_use_effective_values_and_never_rewrite_fragments() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/agents.toml\"]\n");
    home.write("config/agents.toml", "[display]\nidle_animation = true\n");
    let fragment = read(&home, "config/agents.toml");
    assert!(Config::migrate_idle_animation_off_once());
    assert_eq!(read(&home, "config/agents.toml"), fragment);
    assert!(!home.load().unwrap().display.idle_animation);
    assert!(read(&home, "config.toml").contains("idle_animation = false"));
}

#[test]
fn legacy_headed_spawn_mode_still_migrates_to_inline() {
    let home = Home::new();
    home.write(
        "config.toml",
        "[agents]\nswarm_spawn_mode = \"headed\" # legacy\nswarm_max_concurrent_agents = 32\n",
    );
    assert_eq!(
        home.load().unwrap().agents.swarm_spawn_mode,
        SwarmSpawnMode::Visible,
        "headed stays a valid alias of visible"
    );
    assert!(Config::migrate_legacy_swarm_spawn_mode_once());
    let root = read(&home, "config.toml");
    assert!(
        root.contains("swarm_spawn_mode = \"inline\" # legacy"),
        "{root}"
    );
    assert!(root.contains("swarm_max_concurrent_agents = 32"), "{root}");
    assert!(
        home.dir
            .path()
            .join("migrations/swarm-spawn-mode-inline")
            .exists()
    );
}

#[test]
fn migrations_tolerate_an_unrelated_composition_failure() {
    let home = Home::new();
    home.write(
        "config.toml",
        "include = [\"config/missing.toml\"]\n[agents]\nswarm_spawn_mode = \"visible\"\n[display]\nidle_animation = true\n",
    );
    assert!(home.load().is_err(), "composition is broken");
    assert!(Config::migrate_legacy_swarm_spawn_mode_once());
    assert!(Config::migrate_idle_animation_off_once());
    let root = read(&home, "config.toml");
    assert!(
        root.contains("include = [\"config/missing.toml\"]"),
        "{root}"
    );
    assert!(root.contains("swarm_spawn_mode = \"inline\""), "{root}");
    assert!(root.contains("idle_animation = false"), "{root}");

    // A root that is not even TOML is left untouched and retried later.
    home.write("config.toml", "[agents\nswarm_spawn_mode = \"visible\"\n");
    let before = read(&home, "config.toml");
    let _ = std::fs::remove_dir_all(home.dir.path().join("migrations"));
    assert!(!Config::migrate_legacy_swarm_spawn_mode_once());
    assert_eq!(read(&home, "config.toml"), before);
    assert!(
        !home
            .dir
            .path()
            .join("migrations/swarm-spawn-mode-inline")
            .exists(),
        "no marker, so it retries once the file parses"
    );
}

#[test]
fn global_cache_sees_equal_length_fragment_edit_membership_and_recovery() {
    let home = Home::new();
    home.write("config.toml", "include = [\"config/ui.toml\"]\n");
    home.write("config/ui.toml", "[display]\ncentered = true\n");
    assert!(crate::config::config().display.centered);

    // Same length, root untouched.
    home.write("config/ui.toml", "[display]\ncentered = false\n");
    assert!(!crate::config::config().display.centered);

    // Membership: a newly listed fragment takes effect.
    home.write("config/more.toml", "[display]\ncentered = true\n");
    home.write(
        "config.toml",
        "include = [\"config/ui.toml\", \"config/more.toml\"]\n",
    );
    assert!(crate::config::config().display.centered);

    // Deletion: strict reports the source, non-strict is whole-config defaults.
    std::fs::remove_file(home.dir.path().join("config/more.toml")).unwrap();
    let error = format!(
        "{:#}",
        crate::config::config_strict().expect_err("deleted source")
    );
    assert!(error.contains("more.toml"), "{error}");
    assert_eq!(
        crate::config::config().display.centered,
        Config::default().display.centered,
        "non-strict falls back to whole-config defaults, never a partial merge"
    );

    // Restoration recovers the complete composition.
    home.write("config/more.toml", "[display]\ncentered = true\n");
    assert!(crate::config::config_strict().unwrap().display.centered);
}

#[test]
fn trust_revocation_in_a_fragment_is_seen_on_the_next_check() {
    let home = Home::new();
    let prev = std::env::var_os("JCODE_TRUSTED_EXTERNAL_AUTH_SOURCES");
    crate::env::remove_var("JCODE_TRUSTED_EXTERNAL_AUTH_SOURCES");
    home.write("config.toml", "include = [\"config/auth.toml\"]\n");
    home.write(
        "config/auth.toml",
        "[auth]\ntrusted_external_sources = [\"claude_code\"]\n",
    );
    assert!(Config::external_auth_source_allowed("claude_code"));
    // Equal-length edit: same size, only content differs.
    home.write(
        "config/auth.toml",
        "[auth]\ntrusted_external_sources = [\"claude_cod3\"]\n",
    );
    assert!(!Config::external_auth_source_allowed("claude_code"));
    assert!(Config::external_auth_source_allowed("claude_cod3"));
    if let Some(prev) = prev {
        crate::env::set_var("JCODE_TRUSTED_EXTERNAL_AUTH_SOURCES", prev);
    }
}

#[test]
fn invalid_composition_denies_trust_hooks_and_task_policy() {
    let home = Home::new();
    let prev = std::env::var_os("JCODE_TRUSTED_EXTERNAL_AUTH_SOURCES");
    home.write("config.toml", "include = [\"config/policy.toml\"]\n");
    home.write(
        "config/policy.toml",
        "[auth]\ntrusted_external_sources = [\"claude_code\"]\n[hooks]\npre_tool = \"true\"\n",
    );
    assert!(Config::external_auth_source_allowed("claude_code"));
    assert!(!crate::hooks::hook_commands("pre_tool").is_empty());

    std::fs::remove_file(home.dir.path().join("config/policy.toml")).unwrap();
    crate::env::set_var("JCODE_TRUSTED_EXTERNAL_AUTH_SOURCES", "claude_code");
    assert!(
        !Config::external_auth_source_allowed("claude_code"),
        "broken composition must deny even with an env override on top of the fallback"
    );
    match prev {
        Some(prev) => crate::env::set_var("JCODE_TRUSTED_EXTERNAL_AUTH_SOURCES", prev),
        None => crate::env::remove_var("JCODE_TRUSTED_EXTERNAL_AUTH_SOURCES"),
    }
    assert!(crate::hooks::hook_commands("pre_tool").is_empty());
    let outcome =
        futures::executor::block_on(crate::hooks::run_pre_tool_gate("s", None, "bash", "{}"));
    assert!(
        matches!(outcome.decision, crate::hooks::GateDecision::Block { .. }),
        "{outcome:?}"
    );
    let policy_error = format!("{:#}", Config::effective_task_policy().expect_err("strict"));
    assert!(policy_error.contains("policy.toml"), "{policy_error}");

    // Repair recovers on the next check.
    home.write("config/policy.toml", "[hooks]\npre_tool = \"true\"\n");
    assert!(!crate::hooks::hook_commands("pre_tool").is_empty());
    assert!(Config::effective_task_policy().is_ok());
}

const SPLIT_ROOT: &str = "# Nonsecret split-layout fixture (disposable JCODE_HOME).\ninclude = [\"config/ui.toml\", \"config/hooks.toml\"]\n\n[display]\ncentered = false\n";
const SPLIT_UI: &str = "[display]\ncentered = true\nshow_thinking = true\n\n[keybindings]\nscroll_up = \"ctrl+y\"\n\n[gateway]\nport = 7777\n";
const SPLIT_HOOKS: &str = "[hooks]\npre_tool_timeout_ms = 1234\n";
const SINGLE: &str = "[display]\ncentered = false\nshow_thinking = true\n\n[keybindings]\nscroll_up = \"ctrl+y\"\n\n[gateway]\nport = 7777\n\n[hooks]\npre_tool_timeout_ms = 1234\n";

/// Mirrors `scratch/applyall/home-config/{split,single}` (nonsecret).
#[test]
fn split_layout_equals_single_file_and_is_never_discovered_or_rewritten() {
    let split = Home::new();
    split.write("config.toml", SPLIT_ROOT);
    split.write("config/ui.toml", SPLIT_UI);
    split.write("config/hooks.toml", SPLIT_HOOKS);
    // Present on disk but not listed: must never be read.
    split.write(
        "config/unlisted.toml",
        "[display]\ncentered = \"not a bool\"\n",
    );
    split.write(
        "config.d/auto.toml",
        "[display]\ncentered = \"not a bool\"\n",
    );
    let split_cfg = split.load().expect("split layout loads");
    let split_toml = toml::to_string(&split_cfg).unwrap();
    let before: Vec<_> = ["config.toml", "config/ui.toml", "config/hooks.toml"]
        .iter()
        .map(|p| std::fs::read(split.dir.path().join(p)).unwrap())
        .collect();
    let _ = split.load();
    let after: Vec<_> = ["config.toml", "config/ui.toml", "config/hooks.toml"]
        .iter()
        .map(|p| std::fs::read(split.dir.path().join(p)).unwrap())
        .collect();
    assert_eq!(before, after, "loading never rewrites user files");
    drop(split);

    let single = Home::new();
    single.write("config.toml", SINGLE);
    let single_cfg = single.load().expect("single file loads");
    // Root `centered = false` overrides the fragment's `true` in the split layout.
    assert_eq!(split_toml, toml::to_string(&single_cfg).unwrap());
    assert!(!single_cfg.display.centered);
    assert_eq!(single_cfg.hooks.pre_tool_timeout_ms, 1234);
    assert_eq!(single_cfg.gateway.port, 7777);
}

#[test]
fn restart_only_sections_are_labeled_needs_restart() {
    use crate::config::change_report::{Liveness, liveness_for_key};
    assert_eq!(liveness_for_key("gateway.port"), Liveness::NeedsRestart);
    assert_eq!(liveness_for_key("display.centered"), Liveness::Live);
}
