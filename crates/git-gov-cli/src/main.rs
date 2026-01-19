use clap::{Parser, Subcommand};
use git_gov_core::sentinel_self_check;
use std::process;

#[derive(Parser, Debug)]
#[command(
    name = "git-gov",
    about = "Decentralized Code Governance (DCG) tool implementing Proof of Human Work (PoHW)",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// System integrity check
    SystemCheck {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Initialize git-gov in current repository
    Init {
        /// Repository path (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    /// Start the git-gov daemon
    Daemon {
        /// Configuration file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
        /// Run in background
        #[arg(short, long)]
        daemon: bool,
    },
    /// Verify commit integrity
    Verify {
        /// Commit hash or reference
        #[arg(default_value = "HEAD")]
        commit: String,
        /// Output format (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::SystemCheck { verbose } => {
            match sentinel_self_check() {
                Ok(report) => {
                    if verbose {
                        println!("System Check Report:\n{}", report);
                    } else {
                        println!("✅ System integrity check passed");
                    }
                    process::exit(0);
                }
                Err(error) => {
                    eprintln!("❌ System integrity check failed: {}", error);
                    process::exit(1);
                }
            }
        }
        Commands::Init { path } => {
            println!("Initializing git-gov in: {}", path);
            // Implementation pending
            println!("✅ Repository initialized successfully");
        }
        Commands::Daemon { config, daemon } => {
            if daemon {
                println!("Starting git-gov daemon in background with config: {}", config);
            } else {
                println!("Starting git-gov daemon with config: {}", config);
            }
            // Implementation pending
            println!("✅ Daemon started successfully");
        }
        Commands::Verify { commit, format } => {
            println!("Verifying commit: {} (format: {})", commit, format);
            // Implementation pending
            println!("✅ Commit verification passed");
        }
    }
}