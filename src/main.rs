mod cmd;
mod git;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "trk", about = "LLM-maintained wiki for codebases")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create the orphan branch with skeleton wiki files
    Init,
    /// Wire hooks into .claude/settings.local.json
    Setup {
        #[command(subcommand)]
        target: SetupTarget,
    },
    /// Output wiki context for Claude Code injection
    Prime,
}

#[derive(Subcommand)]
enum SetupTarget {
    /// Configure Claude Code hooks and permissions
    Claude,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Init => cmd::init::run(),
        Command::Setup { target } => match target {
            SetupTarget::Claude => cmd::setup_claude::run(),
        },
        Command::Prime => cmd::prime::run(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
