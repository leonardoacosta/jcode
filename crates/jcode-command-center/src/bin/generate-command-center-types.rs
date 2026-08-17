use std::env;
use std::path::PathBuf;

fn main() -> std::io::Result<()> {
    let out_dir = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("apps/command-center/src/generated"));
    jcode_command_center::write_typescript_contract(&out_dir)
}
