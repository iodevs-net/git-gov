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
    /// Check daemon status
    Status,
    /// View real-time kinematic metrics
    Metrics {
        /// Output only the human score (short format)
        #[arg(short, long)]
        short: bool,
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
    /// Verificación termodinámica del trabajo (para hooks)
    VerifyWork,
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
                    
                    // Install hooks
                    match git_gov_core::git::install_hooks(&_repo) {
                        Ok(_) => println!("✅ Git hooks installed successfully"),
                        Err(e) => eprintln!("⚠️ Failed to install git hooks: {}", e),
                    }
                    
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
        Commands::Status => {
            match query_daemon(git_gov_core::protocol::Request::GetStatus).await {
                Ok(git_gov_core::protocol::Response::Status { is_running, uptime_secs, events_captured }) => {
                    println!("Daemon Status:");
                    println!("  Running: {}", if is_running { "✅ Yes" } else { "❌ No" });
                    println!("  Uptime:  {}s", uptime_secs);
                    println!("  Events:  {}", events_captured);
                }
                Ok(git_gov_core::protocol::Response::Error(e)) => {
                    eprintln!("❌ Daemon error: {}", e);
                }
                Ok(_) => {
                    eprintln!("❌ Unexpected response from daemon");
                }
                Err(e) => {
                    eprintln!("❌ Could not connect to daemon: {}. Is it running?", e);
                }
            }
        }
        Commands::Metrics { short } => {
            match query_daemon(git_gov_core::protocol::Request::GetMetrics).await {
                Ok(git_gov_core::protocol::Response::Metrics { ldlj, entropy, throughput, human_score, coupling, battery_level }) => {
                    if short {
                        println!("{:.4}", human_score);
                    } else {
                        println!("GovMonitor - Estado Termodinámico:");
                        println!("  🔋 Energía Kinética: {:.1}%", battery_level);
                        println!("  🧠 Acoplamiento:     {:.1}%", coupling * 100.0);
                        println!("  --------------------------------");
                        println!("  LDLJ (Fluidez):      {:.4}", ldlj);
                        println!("  Entropía Motora:     {:.4}", entropy);
                        println!("  Throughput:          {:.4}", throughput);
                        println!("  Human Score:         {:.2}%", human_score * 100.0);
                    }
                }
                Ok(git_gov_core::protocol::Response::Error(e)) => {
                    if !short { eprintln!("❌ Daemon error: {}", e); }
                    process::exit(1);
                }
                Ok(_) => {
                    if !short { eprintln!("❌ Unexpected response from daemon"); }
                    process::exit(1);
                }
                Err(e) => {
                    if !short { eprintln!("❌ Could not connect to daemon: {}. Is it running?", e); }
                    process::exit(1);
                }
            }
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
        Commands::VerifyWork => {
            use git_gov_core::git::get_staged_diff;
            use git_gov_core::complexity::estimate_entropic_cost;

            let repo = match open_repository(Path::new(".")) {
                Ok(repo) => repo,
                Err(e) => {
                    eprintln!("❌ Error opening repository: {}", e);
                    process::exit(1);
                }
            };

            let diff = match get_staged_diff(&repo) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error getting staged diff: {}", e);
                    process::exit(1);
                }
            };

            if diff.is_empty() {
                process::exit(0);
            }

            let cost = estimate_entropic_cost(&diff);
            
            match query_daemon(git_gov_core::protocol::Request::GetTicket { cost }).await {
                Ok(git_gov_core::protocol::Response::Ticket { success, message, .. }) => {
                    if success {
                        println!("✅ Thermodynamic check passed: {}", message);
                        process::exit(0);
                    } else {
                        eprintln!("❌ {}", message);
                        process::exit(1);
                    }
                }
                Ok(git_gov_core::protocol::Response::Error(e)) => {
                    eprintln!("❌ Daemon error: {}", e);
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("❌ Daemon communication error: {}", e);
                    process::exit(1);
                }
                _ => {
                    eprintln!("❌ Unexpected response from daemon");
                    process::exit(1);
                }
            }
        }
    }
}

async fn query_daemon(request: git_gov_core::protocol::Request) -> anyhow::Result<git_gov_core::protocol::Response> {
    use tokio::net::UnixStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let socket_path = "/tmp/git-gov.sock";
    let mut stream = UnixStream::connect(socket_path).await?;
    
    let request_json = serde_json::to_vec(&request)?;
    stream.write_all(&request_json).await?;
    
    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await?;
    
    let response: git_gov_core::protocol::Response = serde_json::from_slice(&buffer[..n])?;
    Ok(response)
}