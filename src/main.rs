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

    match cli.command {
        Command::Init => {
            eprintln!("trk init: not yet implemented");
            std::process::exit(1);
        }
        Command::Setup { target } => match target {
            SetupTarget::Claude => {
                eprintln!("trk setup claude: not yet implemented");
                std::process::exit(1);
            }
        },
        Command::Prime => {
            eprintln!("trk prime: not yet implemented");
            std::process::exit(1);
        }
    }
}
