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
    /// Write a file to the wiki (content from stdin)
    Write {
        /// Path within the wiki (e.g. pages/architecture.md, toc.md)
        path: String,
        /// Commit message
        #[arg(short, long)]
        message: Option<String>,
    },
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
        Command::Write { path, message } => cmd::write::run(&path, message.as_deref()),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
