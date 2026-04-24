mod cmd;
mod config;
mod git;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "trk", about = "LLM-maintained wiki for codebases")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize trapperkeeper in this repo
    Init {
        /// Store the wiki as a committed directory at PATH instead of
        /// on the `trapperkeeper` orphan branch.
        #[arg(long, value_name = "PATH")]
        in_tree: Option<PathBuf>,
        /// Adopt an existing wiki directory (requires --in-tree). Skips
        /// writing skeleton files that already exist; still sets up the
        /// config, .gitignore, and .gitattributes entries.
        #[arg(long, requires = "in_tree")]
        adopt: bool,
    },
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
        Command::Init { in_tree, adopt } => cmd::init::run(in_tree, adopt),
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
