#[test]
fn artifact_action_destination_builds_direct_command_and_rejects_unsafe_targets() {
    let target = "/tmp/decision brief.md";

    for destination in ["mopen", "ropen", "iopen"] {
        let command = artifact_action_command(destination, target).expect("valid artifact action");
        assert_eq!(command.get_program(), std::ffi::OsStr::new(destination));
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [std::ffi::OsStr::new(target)]
        );
    }

    for target in ["", "  ", "--help", "-rf"] {
        assert!(artifact_action_command("mopen", target).is_none());
    }
}

#[test]
fn brief_aloud_builds_direct_say_brief_command_and_rejects_blank_prose() {
    let prose = "The decision is approved because it keeps delivery focused.";
    let command = brief_aloud_command(prose).expect("non-blank spoken prose");

    assert_eq!(command.get_program(), std::ffi::OsStr::new("say_brief"));
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        [std::ffi::OsStr::new(prose)]
    );
    assert!(brief_aloud_command("  ").is_none());
}
