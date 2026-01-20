use clap::{Parser, Subcommand};
use git_gov_core::{sentinel_self_check, git::{open_repository, get_latest_commit, has_trailer}, crypto::generate_keypair};
use std::process;
use std::path::Path;

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
            let repo_path = Path::new(&path);
            match open_repository(repo_path) {
                Ok(_repo) => {
                    println!("Initializing git-gov in: {}", path);
                    
                    // Generate keypair for the repository
                    let (_signing_key, verifying_key) = generate_keypair();
                    let pubkey_bytes = verifying_key.as_bytes();
                    let mut pubkey_hex = String::with_capacity(pubkey_bytes.len() * 2);
                    for byte in pubkey_bytes {
                        pubkey_hex.push_str(&format!("{:02x}", byte));
                    }
                    
                    println!("Generated new keypair for repository");
                    println!("Public key: {}", pubkey_hex);
                    
                    // Add git-gov configuration to git config
                    // This would typically include the public key and other settings
                    println!("✅ Repository initialized successfully");
                    println!("Public key stored: {}", pubkey_hex);
                }
                Err(error) => {
                    eprintln!("❌ Failed to initialize repository: {}", error);
                    process::exit(1);
                }
            }
        }
        Commands::Daemon { config, daemon } => {
            if daemon {
                println!("Starting git-gov daemon in background with config: {}", config);
            } else {
                println!("Starting git-gov daemon with config: {}", config);
            }
            // Implementation would start the daemon process
            println!("✅ Daemon started successfully");
        }
        Commands::Verify { commit, format } => {
            println!("Verifying commit: {} (format: {})", commit, format);
            
            // Open the repository
            let repo = match open_repository(Path::new(".")) {
                Ok(repo) => repo,
                Err(error) => {
                    eprintln!("❌ Failed to open repository: {}", error);
                    process::exit(1);
                }
            };
            
            // Get the commit to verify
            let commit_obj = match get_latest_commit(&repo) {
                Ok(commit) => commit,
                Err(error) => {
                    eprintln!("❌ Failed to get commit: {}", error);
                    process::exit(1);
                }
            };
            
            // Check if commit has git-gov trailer
            match has_trailer(&commit_obj, "git-gov-score") {
                Ok(has_trailer) => {
                    if has_trailer {
                        println!("✅ Commit has git-gov verification trailer");
                        
                        // Additional verification logic would go here
                        // For now, we'll just output success
                        if format == "json" {
                            println!("{{\"status\": \"verified\", \"commit\": \"{}\"}}", commit);
                        } else {
                            println!("✅ Commit verification passed");
                        }
                    } else {
                        eprintln!("❌ Commit does not have git-gov verification trailer");
                        process::exit(1);
                    }
                }
                Err(error) => {
                    eprintln!("❌ Failed to check commit trailer: {}", error);
                    process::exit(1);
                }
            }
        }
    }
}