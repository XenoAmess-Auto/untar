use std::env;
use std::fs;

use clap::CommandFactory;
use clap_complete::Shell;

include!("src/cli.rs");

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let mut cmd = Args::command();

    let completions_dir = std::path::PathBuf::from(&out_dir).join("completions");
    fs::create_dir_all(&completions_dir).expect("failed to create completions dir");

    for shell in [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Elvish,
    ] {
        clap_complete::generate_to(shell, &mut cmd, "untar", &completions_dir)
            .expect("failed to generate shell completion");
    }

    let man_dir = std::path::PathBuf::from(&out_dir).join("man");
    fs::create_dir_all(&man_dir).expect("failed to create man dir");

    let man = clap_mangen::Man::new(cmd);
    let mut buffer = Vec::new();
    man.render(&mut buffer).expect("failed to render man page");
    fs::write(man_dir.join("untar.1"), buffer).expect("failed to write man page");
}
