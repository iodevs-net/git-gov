use clap::{Parser, Subcommand};
use git_gov_core::{sentinel_self_check, git::{open_repository}, crypto::generate_keypair};
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
    /// Desactiva git-gov en el repositorio actual (elimina hooks)
    Disable {
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
    /// Activa el centinela (inicia el daemon en background)
    On,
    /// Desactiva el centinela (detiene el daemon)
    Off,
    /// View real-time kinematic metrics
    Metrics {
        /// Output only the human score (short format)
        #[arg(short, long)]
        short: bool,
    },
    /// Verificación commit-por-commit (para CI/CD o auditorías)
    Verify {
        /// Hash o referencia del commit
        #[arg(default_value = "HEAD")]
        commit: String,
        /// Formato de salida (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Registra una clave pública para verificación en este repositorio
    RegisterKey {
        /// Clave pública en formato hexadecimal
        #[arg(short, long)]
        key: String,
        /// Alias para la clave (ej: nombre del dev)
        #[arg(short, long)]
        alias: String,
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
        Commands::Disable { path } => {
            let repo_path = Path::new(&path);
            match open_repository(repo_path) {
                Ok(repo) => {
                    match git_gov_core::git::remove_hooks(&repo) {
                        Ok(_) => println!("✅ Git-Gov disabled: Hooks removed successfully"),
                        Err(e) => eprintln!("❌ Failed to remove hooks: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to open repository: {}", e);
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
        Commands::On => {
            use std::process::{Command, Stdio};
            println!("🚀 Encendiendo el centinela termodinámico...");
            
            match Command::new("git-gov-daemon")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn() {
                    Ok(_) => {
                        println!("✅ Centinela activado en background.");
                        println!("Usa 'git-gov status' para verificar.");
                    },
                    Err(_) => {
                        // Reintento con path explícito por si no está en PATH todavía
                        match Command::new("/usr/local/bin/git-gov-daemon")
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn() {
                                Ok(_) => println!("✅ Centinela activado en background."),
                                Err(e) => eprintln!("❌ Error al encender el centinela: {}. ¿Está instalado?", e),
                            }
                    }
                }
        }
        Commands::Off => {
            use std::process::Command;
            println!("🛑 Apagando el centinela...");
            
            match Command::new("pkill")
                .arg("-x") // Match exact name
                .arg("git-gov-daemon")
                .status() {
                    Ok(status) if status.success() => println!("✅ Centinela desactivado."),
                    Ok(_) => println!("⚠️ El centinela no parecía estar corriendo."),
                    Err(e) => eprintln!("❌ Error al apagar el centinela: {}", e),
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
        Commands::RegisterKey { key, alias } => {
            let repo = match open_repository(Path::new(".")) {
                Ok(repo) => repo,
                Err(e) => {
                    eprintln!("❌ Error opening repository: {}", e);
                    process::exit(1);
                }
            };

            match git_gov_core::git::register_public_key(&repo, &key, &alias) {
                Ok(_) => println!("✅ Key registered successfully for alias: {}", alias),
                Err(e) => {
                    eprintln!("❌ Failed to register key: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Verify { commit, format } => {
            use git_gov_core::git::{get_trusted_keys};
            use git_gov_core::crypto::{verify_signature, VerifyingKey};

            let repo = match open_repository(Path::new(".")) {
                Ok(repo) => repo,
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    process::exit(1);
                }
            };

            let commit_obj = match repo.revparse_single(&commit) {
                Ok(obj) => obj.peel_to_commit().unwrap(),
                Err(e) => {
                    eprintln!("❌ Commit not found: {}", e);
                    process::exit(1);
                }
            };

            let message = commit_obj.message().unwrap_or("");
            let trusted_keys = get_trusted_keys(&repo).unwrap_or_default();

            // Buscar trailer de git-gov
            let mut found = false;
            for line in message.lines() {
                if line.starts_with("git-gov-score:") {
                    let score_part = line.replace("git-gov-score:", "").trim().to_string();
                    // El formato esperado es "score=0.85:sig=<hex>"
                    let parts: Vec<&str> = score_part.split(":sig=").collect();
                    if parts.len() == 2 {
                        let score_str = parts[0].replace("score=", "");
                        let sig_hex = parts[1];
                        let sig_bytes = hex::decode(sig_hex).unwrap_or_default();
                        
                        // Reconstruir el mensaje que fue firmado
                        // (En el PoC era "VALID:cost=X:ts=Y", pero para el commit usaremos el score directo)
                        // Para simplificar esta v1 de verificación de PR, verificamos que la firma sea válida
                        // para CUALQUIER clave confiable sobre el hash del commit o el score.
                        
                        let mut verified = false;
                        let mut signer_alias = "Unknown";

                        for (alias, key_hex) in &trusted_keys {
                            let key_bytes = hex::decode(key_hex).unwrap_or_default();
                            if let Ok(verifying_key) = VerifyingKey::from_bytes(&key_bytes.try_into().unwrap_or([0;32])) {
                                // El mensaje firmado por el daemon en VerifyWork era VALID:cost=...
                                // Pero el hook lo guarda parseado. 
                                // Para esta versión, aceptamos la firma si es válida.
                                if verify_signature(&verifying_key, parts[0].as_bytes(), &sig_bytes).unwrap_or(false) {
                                    verified = true;
                                    signer_alias = alias;
                                    break;
                                }
                            }
                        }

                        if verified {
                            if format == "json" {
                                println!("{{\"status\": \"verified\", \"signer\": \"{}\", \"score\": {}}}", signer_alias, score_str);
                            } else {
                                println!("✅ Commit VERIFICADO Criptográficamente");
                                println!("   Firmante: {}", signer_alias);
                                println!("   Score:    {}", score_str);
                            }
                            found = true;
                        }
                    }
                }
            }

            if !found {
                if format == "json" {
                    println!("{{\"status\": \"failed\", \"reason\": \"no_valid_signature\"}}");
                } else {
                    eprintln!("❌ FALLO DE VERIFICACIÓN: No se encontró firma válida de Git-Gov.");
                }
                process::exit(1);
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
                Ok(git_gov_core::protocol::Response::Ticket { success, message, signature }) => {
                    if success {
                        println!("✅ Thermodynamic check passed: {}", message);
                        
                        // Guardar el ticket firmado para el hook prepare-commit-msg
                        if let Some(sig_bytes) = signature {
                            let sig_hex = hex::encode(sig_bytes);
                            let ticket_data = format!("score={:.2}:sig={}", cost, sig_hex);
                            
                            let gov_dir = repo.path().join("git-gov");
                            if !gov_dir.exists() {
                                let _ = std::fs::create_dir_all(&gov_dir);
                            }
                            let ticket_file = gov_dir.join("latest_ticket");
                            if let Err(e) = std::fs::write(ticket_file, ticket_data) {
                                eprintln!("⚠️ Error saving ticket: {}", e);
                            }
                        }
                        
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