use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process;

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    append_log(&args);
    let scenario = fs::read_to_string(".fake-orca-scenario")
        .expect("fake Orca scenario")
        .trim()
        .to_string();

    match args.as_slice() {
        [a, b, c, ..] if a == "worktree" && b == "current" && c == "--json" => {
            let path = env::current_dir().expect("current directory");
            success(&format!(
                "{{\"worktree\":{{\"id\":\"worktree-1\",\"repoId\":\"repo-1\",\"hostId\":\"host-1\",\"path\":{}}}}}",
                json_string(&path.to_string_lossy())
            ));
        }
        [a, b, c, ..] if a == "repo" && b == "list" && c == "--json" => {
            let path = env::current_dir().expect("current directory");
            success(&format!(
                "{{\"repos\":[{{\"id\":\"repo-1\",\"path\":{}}}]}}",
                json_string(&path.to_string_lossy())
            ));
        }
        [a, b, c, ..] if a == "project" && b == "setups" && c == "--json" => success(
            "{\"setups\":[{\"id\":\"setup-1\",\"projectId\":\"project-1\",\"repoId\":\"repo-1\",\"hostId\":\"host-1\",\"setupState\":\"ready\"}]}",
        ),
        [a, b, c, ..] if a == "project" && b == "list" && c == "--json" => {
            success("{\"projects\":[{\"id\":\"project-1\",\"sourceRepoIds\":[\"repo-1\"]}]}")
        }
        [a, b, ..] if a == "terminal" && b == "show" => {
            success("{\"terminal\":{\"handle\":\"terminal-coordinator\"}}")
        }
        [a, b, ..] if a == "orchestration" && b == "run-create" => success(
            "{\"run\":{\"id\":\"run-1\",\"objective\":\"acceptance\"},\"binding\":{\"consumerGeneration\":1}}",
        ),
        [a, b, ..] if a == "orchestration" && b == "run-current" => {
            let run = if matches!(scenario.as_str(), "retry" | "rejected") {
                "run-other"
            } else {
                "run-1"
            };
            success(&format!("{{\"run\":{{\"id\":{}}}}}", json_string(run)));
        }
        [a, b, ..] if a == "orchestration" && b == "run-use" => success(
            "{\"run\":{\"id\":\"run-1\",\"objective\":\"acceptance\"},\"binding\":{\"consumerGeneration\":2}}",
        ),
        [a, b, ..] if a == "orchestration" && b == "task-create" => success(
            "{\"task\":{\"id\":\"task-1\",\"run_id\":\"run-1\",\"display_name\":\"acceptance\",\"status\":\"pending\"}}",
        ),
        [a, b, ..] if a == "orchestration" && b == "task-list" => {
            success("{\"tasks\":[{\"id\":\"task-1\",\"status\":\"failed\"}]}")
        }
        [a, b, ..] if a == "orchestration" && b == "worker-list" => {
            success("{\"workers\":[{\"taskId\":\"task-1\",\"dispatchId\":\"dispatch-prior\"}]}")
        }
        [a, b, ..] if a == "orchestration" && b == "worker-start" => worker_start(&args, &scenario),
        [a, b, ..] if a == "orchestration" && b == "worker-show" => worker_show(&args, &scenario),
        [a, b, ..] if a == "orchestration" && b == "worker-stop" => worker_stop(&args, &scenario),
        [a, b, ..] if a == "orchestration" && b == "worker-abandon" => worker_abandon(&args),
        [a, b, ..] if a == "orchestration" && b == "worker-release" => {
            worker_release(&args, &scenario)
        }
        _ => fail(&format!("unsupported fake Orca command: {args:?}")),
    }
}

fn worker_start(args: &[String], scenario: &str) -> ! {
    if arg_value(args, "--retry-of").is_some() {
        fs::write(".fake-orca-worker-state", "ready").unwrap();
        worker_start_receipt("dispatch-retry", "ready", 0);
    }
    match scenario {
        "ready" => {
            fs::write(".fake-orca-worker-state", "ready").unwrap();
            worker_start_receipt("dispatch-start", "ready", 0)
        }
        "failed" => {
            fs::write(".fake-orca-worker-state", "failed").unwrap();
            worker_start_receipt("dispatch-start", "failed", 1)
        }
        "outcome_unknown" | "restart" => {
            fs::write(".fake-orca-worker-state", "start_unknown").unwrap();
            println!(
                "{{\"id\":\"fake-response\",\"ok\":false,\"result\":null,\"error\":{{\"code\":\"operation_unknown\",\"message\":\"accepted but unsettled\",\"data\":{{\"dispatchId\":\"dispatch-start\"}}}},\"_meta\":{{\"runtimeId\":\"runtime-fake\"}}}}"
            );
            process::exit(1);
        }
        other => fail(&format!("worker-start unsupported for scenario {other}")),
    }
}

fn worker_start_receipt(dispatch: &str, state: &str, exit: i32) -> ! {
    print_envelope(
        &format!(
            "{{\"runId\":\"run-1\",\"taskId\":\"task-1\",\"dispatchId\":{},\"state\":{},\"stage\":{},\"setup\":{{}},\"launch\":{{}},\"effects\":[{{\"kind\":\"terminal\",\"role\":\"agent\",\"action\":\"created\",\"id\":\"terminal-worker\",\"surface\":\"background\"}}],\"residualResources\":[]}}",
            json_string(dispatch),
            json_string(state),
            json_string(state)
        ),
        exit,
    )
}

fn worker_show(args: &[String], scenario: &str) -> ! {
    let dispatch = arg_value(args, "--dispatch").expect("worker-show dispatch");
    let state = if dispatch == "dispatch-prior" {
        if scenario == "rejected" {
            "ready"
        } else {
            "failed"
        }
        .to_string()
    } else if dispatch == "dispatch-retry" {
        "ready".to_string()
    } else if scenario == "restart" {
        let count = read_count(".fake-orca-worker-show-count");
        fs::write(".fake-orca-worker-show-count", (count + 1).to_string()).unwrap();
        if count == 0 {
            "start_unknown".to_string()
        } else {
            fs::write(".fake-orca-worker-state", "ready").unwrap();
            "ready".to_string()
        }
    } else {
        fs::read_to_string(".fake-orca-worker-state").unwrap_or_else(|_| "ready".into())
    };
    success(&format!(
        "{{\"dispatch\":{{\"id\":{},\"run_id\":\"run-1\",\"task_id\":\"task-1\"}},\"worker\":{{\"dispatch_id\":{},\"runtime_epoch\":\"runtime-fake\",\"state\":{},\"stage\":{},\"worktree_id\":\"worktree-1\",\"agent_terminal_handle\":\"terminal-worker\",\"effects\":[],\"residualResources\":[],\"startOptions\":{{\"resolvedWorktreeId\":\"worktree-1\",\"agent\":\"codex\"}}}},\"terminal\":null,\"observation\":{{\"status\":\"observed\",\"exactWorker\":true}},\"terminalResource\":null}}",
        json_string(dispatch),
        json_string(dispatch),
        json_string(state.trim()),
        json_string(state.trim())
    ));
}

fn worker_stop(args: &[String], scenario: &str) -> ! {
    let dispatch = arg_value(args, "--dispatch").expect("worker-stop dispatch");
    let (state, exit) = if scenario == "abandon" {
        ("stop_unknown", 1)
    } else {
        ("stopped", 0)
    };
    fs::write(".fake-orca-worker-state", state).unwrap();
    print_envelope(
        &format!(
            "{{\"dispatchId\":{},\"state\":{},\"alreadySettled\":false,\"processAction\":{}}}",
            json_string(dispatch),
            json_string(state),
            json_string(state)
        ),
        exit,
    );
}

fn worker_abandon(args: &[String]) -> ! {
    let dispatch = arg_value(args, "--dispatch").expect("worker-abandon dispatch");
    fs::write(".fake-orca-worker-state", "abandoned").unwrap();
    success(&format!(
        "{{\"dispatchId\":{},\"state\":\"abandoned\",\"alreadySettled\":false,\"stale\":false,\"processAction\":\"abandoned\",\"residualResources\":[{{\"kind\":\"terminal\",\"role\":\"agent\",\"action\":\"retained\",\"id\":\"terminal-worker\",\"surface\":\"background\"}}]}}",
        json_string(dispatch)
    ));
}

fn worker_release(args: &[String], scenario: &str) -> ! {
    let dispatch = arg_value(args, "--dispatch").expect("worker-release dispatch");
    let state = match scenario {
        "release_pending" => "release_pending",
        "release_unknown" => "release_unknown",
        _ => "released",
    };
    let exit = if state == "release_unknown" { 1 } else { 0 };
    print_envelope(
        &format!(
            "{{\"dispatchId\":{},\"state\":{},\"reason\":\"acceptance\",\"lastError\":{}}}",
            json_string(dispatch),
            json_string(state),
            if state == "released" {
                "null".into()
            } else {
                json_string(state)
            }
        ),
        exit,
    );
}

fn success(result: &str) -> ! {
    print_envelope(result, 0)
}

fn print_envelope(result: &str, exit: i32) -> ! {
    println!(
        "{{\"id\":\"fake-response\",\"ok\":true,\"result\":{result},\"error\":null,\"_meta\":{{\"runtimeId\":\"runtime-fake\"}}}}"
    );
    process::exit(exit);
}

fn fail(message: &str) -> ! {
    eprintln!("{message}");
    process::exit(2);
}

fn append_log(args: &[String]) {
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(".fake-orca-log")
        .expect("fake Orca log");
    writeln!(log, "{}", args.join("\t")).expect("append fake Orca log");
}

fn arg_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

fn read_count(path: impl AsRef<Path>) -> usize {
    fs::read_to_string(path)
        .ok()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or(0)
}

fn json_string(value: &str) -> String {
    let mut rendered = String::with_capacity(value.len() + 2);
    rendered.push('"');
    for character in value.chars() {
        match character {
            '"' => rendered.push_str("\\\""),
            '\\' => rendered.push_str("\\\\"),
            '\n' => rendered.push_str("\\n"),
            '\r' => rendered.push_str("\\r"),
            '\t' => rendered.push_str("\\t"),
            other => rendered.push(other),
        }
    }
    rendered.push('"');
    rendered
}
