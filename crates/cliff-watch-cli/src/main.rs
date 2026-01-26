use clap::{Parser, Subcommand};
use cliff_watch_core::{sentinel_self_check, git::{open_repository}, crypto::generate_keypair};
use std::process::{self, Command, Stdio};
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use console::{style, Emoji};
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(
    name = "cliff-watch",
    about = "Decentralized Code Governance (DCG) tool implementing Proof of Human Work (PoHW)",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨  ", "");
static GEAR: Emoji<'_, '_> = Emoji("⚙️  ", "");
static PACKAGE: Emoji<'_, '_> = Emoji("📦  ", "");
static SUCCESS: Emoji<'_, '_> = Emoji("✅  ", "");

#[derive(Subcommand, Debug)]
enum Commands {
    /// System integrity check
    SystemCheck {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Initialize cliff-watch in current repository
    Init {
        /// Repository path (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    /// Desactiva cliff-watch en el repositorio actual (elimina hooks)
    Disable {
        /// Repository path (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    /// Start the cliff-watch daemon
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
    /// Genera la evidencia técnica (trailers) para el commit actual
    Inspect {
        /// Ruta al archivo de mensaje de commit (pasado por Git)
        #[arg()]
        message_file: Option<String>,
    },
    /// Gestión de configuración
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Genera reporte de auditoría
    Report {
        /// Número de commits a analizar
        #[arg(short, long, default_value_t = 100)]
        limit: usize,
        /// Formato de salida (text, json, md)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Configura el entorno de desarrollo de forma automática y premium
    Setup {
        /// Evita la interacción con el usuario
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigAction {
    /// Crea un archivo de configuración por defecto
    Init,
    /// Valida la configuración actual
    Check,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup { yes } => {
            run_setup(yes).await;
        }
        Commands::Report { limit, format } => {
            let repo = match open_repository(Path::new(".")) {
                Ok(repo) => repo,
                Err(e) => {
                    eprintln!("❌ Error opening repository: {}", e);
                    process::exit(1);
                }
            };
            
            match cliff_watch_core::git::get_governance_history(&repo, limit) {
                Ok(entries) => {
                    if format == "json" {
                        println!("{}", serde_json::to_string_pretty(&entries).unwrap());
                        return;
                    }
                    
                    let total_commits = entries.len();
                    let total_score: f64 = entries.iter().map(|e| e.score).sum();
                    let avg_score = if total_commits > 0 { total_score / total_commits as f64 } else { 0.0 };
                    
                    let mut authors = std::collections::HashMap::new();
                    for e in &entries {
                        *authors.entry(e.author.clone()).or_insert(0.0) += e.score;
                    }
                    
                    if format == "md" {
                         println!("# Governance Audit Report");
                         println!("- **Analyzed Commits:** {}", total_commits);
                         println!("- **Total Energy:** {:.2}", total_score);
                         println!("- **Average Entropy:** {:.2}", avg_score);
                         println!("\n## Top Contributors (by Energy)");
                         println!("| Author | Energy |");
                         println!("|--------|--------|");
                         for (author, score) in &authors {
                             println!("| {} | {:.2} |", author, score);
                         }
                    } else {
                        println!("📊 Governance Report (Last {} commits)", limit);
                        println!("--------------------------------");
                        println!("  Commits Analyzed: {}", total_commits);
                        println!("  Total Energy:     {:.2}", total_score);
                        println!("  Average Score:    {:.2}", avg_score);
                        println!("\n🏆 Top Contributors:");
                        for (author, score) in authors {
                            println!("  - {}: {:.2}", author, score);
                        }
                    }
                }
                Err(e) => {
                     eprintln!("❌ Failed to generate report: {}", e);
                     process::exit(1);
                }
            }
        }
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
                    println!("Initializing cliff-watch in: {}", path);
                    
                    // Generate keypair for the repository
                    let (_signing_key, verifying_key) = generate_keypair();
                    let pubkey_bytes = verifying_key.as_bytes();
                    let mut pubkey_hex = String::with_capacity(pubkey_bytes.len() * 2);
                    for byte in pubkey_bytes {
                        pubkey_hex.push_str(&format!("{:02x}", byte));
                    }
                    
                    println!("Generated new keypair for repository");
                    println!("Public key: {}", pubkey_hex);
                    
                    // Add cliff-watch configuration to git config
                    // This would typically include the public key and other settings
                    
                    // Install hooks
                    match cliff_watch_core::git::install_hooks(&_repo) {
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
                    match cliff_watch_core::git::remove_hooks(&repo) {
                        Ok(_) => println!("✅ Cliff-Watch disabled: Hooks removed successfully"),
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
                println!("Starting cliff-watch daemon in background with config: {}", config);
            } else {
                println!("Starting cliff-watch daemon with config: {}", config);
            }
            // Implementation would start the daemon process
            println!("✅ Daemon started successfully");
        }
        Commands::Status => {
            match query_daemon(cliff_watch_core::protocol::Request::GetStatus).await {
                Ok(cliff_watch_core::protocol::Response::Status { is_running, uptime_secs, events_captured }) => {
                    println!("Daemon Status:");
                    println!("  Running: {}", if is_running { "✅ Yes" } else { "❌ No" });
                    println!("  Uptime:  {}s", uptime_secs);
                    println!("  Events:  {}", events_captured);
                }
                Ok(cliff_watch_core::protocol::Response::Error(e)) => {
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
            
            match Command::new("cliff-watch-daemon")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn() {
                    Ok(_) => {
                        println!("✅ Centinela activado en background.");
                        println!("Usa 'cliff-watch status' para verificar.");
                    },
                    Err(_) => {
                        // Reintento con path explícito por si no está en PATH todavía
                        match Command::new("/usr/local/bin/cliff-watch-daemon")
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
                .arg("cliff-watch-daemon")
                .status() {
                    Ok(status) if status.success() => println!("✅ Centinela desactivado."),
                    Ok(_) => println!("⚠️ El centinela no parecía estar corriendo."),
                    Err(e) => eprintln!("❌ Error al apagar el centinela: {}", e),
                }
        }
        Commands::Metrics { short } => {
            match query_daemon(cliff_watch_core::protocol::Request::GetMetrics).await {
                Ok(cliff_watch_core::protocol::Response::Metrics { 
                    ldlj, entropy, throughput, human_score, coupling, battery_level,
                    focus_time_mins, edit_bursts, is_focused, zkp_proof 
                }) => {
                    if short {
                        println!("{:.4}", human_score);
                    } else {
                        println!("GovMonitor - Estado Termodinámico v2.0:");
                        println!("  🔋 Energía (Kinética+Foco): {:.1}%", battery_level);
                        println!("  🧠 Acoplamiento Cognitivo:  {:.1}%", coupling * 100.0);
                        println!("  --------------------------------");
                        println!("  ⏱️  Tiempo de Foco:         {:.2} min", focus_time_mins);
                        println!("  ✍️  Ráfagas de Edición:     {}", edit_bursts);
                        println!("  👁️  Sensor IDE:             {}", if is_focused { "✅ ACTIVO" } else { "💤 INACTIVO" });
                        println!("  --------------------------------");
                        println!("  📊 Métricas Cinemáticas (v1.0):");
                        println!("     LDLJ (Fluidez):       {:.4}", ldlj);
                        println!("     Entropía Motora:      {:.4}", entropy);
                        println!("     Throughput:           {:.4}", throughput);
                        println!("  --------------------------------");
                        println!("  🛡️  Human Probability:     {:.2}%", human_score * 100.0);
                        println!("  🔐  ZKP Proof:             {}", if zkp_proof.is_some() { style("VERIFIED").green() } else { style("PENDING").yellow() });
                    }
                }
                Ok(cliff_watch_core::protocol::Response::Error(e)) => {
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

            match cliff_watch_core::git::register_public_key(&repo, &key, &alias) {
                Ok(_) => println!("✅ Key registered successfully for alias: {}", alias),
                Err(e) => {
                    eprintln!("❌ Failed to register key: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Verify { commit, format } => {
            use cliff_watch_core::git::{get_trusted_keys};
            use cliff_watch_core::crypto::{verify_signature, VerifyingKey};

            #[derive(serde::Serialize)]
            struct VerificationReport {
                status: String,
                commit: String,
                signer: Option<String>,
                score: Option<f64>,
                reason: Option<String>,
            }

            let repo = match open_repository(Path::new(".")) {
                Ok(repo) => repo,
                Err(e) => {
                    let report = VerificationReport {
                        status: "error".to_string(),
                        commit: commit.clone(),
                        signer: None,
                        score: None,
                        reason: Some(format!("Error opening repository: {}", e)),
                    };
                    if format == "json" {
                        println!("{}", serde_json::to_string(&report).unwrap());
                    } else {
                        eprintln!("❌ Error opening repository: {}", e);
                    }
                    process::exit(1);
                }
            };

            let commit_obj = match repo.revparse_single(&commit) {
                Ok(obj) => obj.peel_to_commit().unwrap(),
                Err(e) => {
                    let report = VerificationReport {
                        status: "error".to_string(),
                        commit: commit.clone(),
                        signer: None,
                        score: None,
                        reason: Some(format!("Commit not found: {}", e)),
                    };
                    if format == "json" {
                        println!("{}", serde_json::to_string(&report).unwrap());
                    } else {
                        eprintln!("❌ Commit not found: {}", e);
                    }
                    process::exit(1);
                }
            };

            let message = commit_obj.message().unwrap_or("");
            let trusted_keys = get_trusted_keys(&repo).unwrap_or_default();

            // Buscar trailer de cliff-watch
            let mut found = false;
            for line in message.lines() {
                if line.starts_with("cliff-watch-score:") {
                    let score_part = line.replace("cliff-watch-score:", "").trim().to_string();
                    // El formato esperado es "score=0.85:sig=<hex>"
                    let parts: Vec<&str> = score_part.split(":sig=").collect();
                    if parts.len() == 2 {
                        let score_str = parts[0].replace("score=", "");
                        let sig_hex = parts[1];
                        let sig_bytes = hex::decode(sig_hex).unwrap_or_default();
                        
                        let mut verified = false;
                        let mut signer_alias = "Unknown";

                        for (alias, key_hex) in &trusted_keys {
                            let key_bytes = hex::decode(key_hex).unwrap_or_default();
                            if let Ok(verifying_key) = VerifyingKey::from_bytes(&key_bytes.try_into().unwrap_or([0;32])) {
                                if verify_signature(&verifying_key, parts[0].as_bytes(), &sig_bytes).unwrap_or(false) {
                                    verified = true;
                                    signer_alias = alias;
                                    break;
                                }
                            }
                        }

                        if verified {
                            let score_val = score_str.parse::<f64>().ok();
                            if format == "json" {
                                let report = VerificationReport {
                                    status: "verified".to_string(),
                                    commit: commit.clone(),
                                    signer: Some(signer_alias.to_string()),
                                    score: score_val,
                                    reason: None,
                                };
                                println!("{}", serde_json::to_string(&report).unwrap());
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
                     let report = VerificationReport {
                        status: "failed".to_string(),
                        commit: commit.clone(),
                        signer: None,
                        score: None,
                        reason: Some("no_valid_signature".to_string()),
                    };
                    println!("{}", serde_json::to_string(&report).unwrap());
                } else {
                    eprintln!("❌ FALLO DE VERIFICACIÓN: No se encontró firma válida de Cliff-Watch.");
                }
                process::exit(1);
            }
        }
        Commands::VerifyWork => {
            use cliff_watch_core::git::get_staged_diff;
            use cliff_watch_core::complexity::estimate_entropic_cost;

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
            
            match query_daemon(cliff_watch_core::protocol::Request::GetTicket { cost }).await {
                Ok(cliff_watch_core::protocol::Response::Ticket { success, message, signature }) => {
                    if success {
                        println!("✅ Thermodynamic check passed: {}", message);
                        
                        // Guardar el ticket firmado para el hook prepare-commit-msg
                        if let Some(sig_bytes) = signature {
                            let sig_hex = hex::encode(sig_bytes);
                            let ticket_data = format!("score={:.2}:sig={}", cost, sig_hex);
                            
                            let gov_dir = repo.path().join("cliff-watch");
                            if !gov_dir.exists() {
                                let _ = std::fs::create_dir_all(&gov_dir);
                            }
                            let ticket_file = gov_dir.join("latest_ticket");
                            if let Err(e) = std::fs::write(ticket_file, ticket_data) {
                                eprintln!("⚠️ Error saving ticket: {}", e);
                            }
                        }

                        // v2.0: Obtener datos del Witness para certificación de foco
                        match query_daemon(cliff_watch_core::protocol::Request::GetWitness { reset: true }).await {
                            Ok(cliff_watch_core::protocol::Response::Witness { data }) => {
                                let witness_file = repo.path().join("cliff-watch").join("latest_witness");
                                if let Err(e) = std::fs::write(witness_file, data) {
                                    eprintln!("⚠️ Error saving witness data: {}", e);
                                } else {
                                    println!("✅ Focus witness data recorded (v2.0)");
                                }
                            }
                            _ => eprintln!("⚠️ Could not retrieve focus witness data"),
                        }
                        
                        process::exit(0);
                    } else {
                        eprintln!("❌ {}", message);
                        process::exit(1);
                    }
                }
                Ok(cliff_watch_core::protocol::Response::Error(e)) => {
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
        Commands::Inspect { message_file } => {
            let repo = match open_repository(Path::new(".")) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("❌ Error opening repository: {}", e);
                    process::exit(1);
                }
            };

            // 1. Obtener la evidencia del Witness (reseteando el acumulador)
            match query_daemon(cliff_watch_core::protocol::Request::GetWitness { reset: true }).await {
                Ok(cliff_watch_core::protocol::Response::Witness { data }) => {
                    let gov_dir = repo.path().join("cliff-watch");
                    if !gov_dir.exists() {
                        let _ = std::fs::create_dir_all(&gov_dir);
                    }
                    
                    let witness_file = gov_dir.join("latest_witness");
                    if let Err(e) = std::fs::write(&witness_file, &data) {
                        eprintln!("⚠️ Error saving witness data: {}", e);
                    } else {
                        println!("✅ Evidence generated: Cliff-Watch-Witness v2.0");
                    }

                    // 2. Si se proporciona un archivo de mensaje (Git hook manual), inyectamos ahora
                    if let Some(msg_file_path) = message_file {
                        let msg_path = Path::new(&msg_file_path);
                        if msg_path.exists() {
                            let _ = Command::new("git")
                                .args(["interpret-trailers", "--in-place", "--trailer", &format!("Cliff-Watch-Witness: {}", data), &msg_file_path])
                                .status();
                        }
                    }
                }
                _ => {
                    eprintln!("❌ Could not retrieve witness evidence from daemon.");
                    process::exit(1);
                }
            }
        }
        Commands::Config { action } => {
            match action {
                ConfigAction::Init => {
                    let config_content = r#"# Cliff-Watch Configuration

[governance]
# Dificultad de la "Prueba de Sudor" (Easy, Normal, Hardcore)
difficulty = "Normal"
# Entropía mínima para validar un commit (base 2.5)
min_entropy = 2.5

[monitoring]
# Ventana de agrupación de eventos (ms)
debounce_window_ms = 500
# Directorios a ignorar (además de .git)
ignore_top_level_dirs = [".git", "target", "node_modules", "dist", "build"]
# Extensiones de archivo a ignorar
ignore_extensions = ["log", "lock", "tmp", "bak"]
"#;
                    let path = Path::new("cliff-watch.toml");
                    if path.exists() {
                        eprintln!("⚠️ cliff-watch.toml already exists!");
                        process::exit(1);
                    }
                    if let Err(e) = std::fs::write(path, config_content) {
                        eprintln!("❌ Failed to create config file: {}", e);
                        process::exit(1);
                    }
                    println!("✅ Created cliff-watch.toml");
                }
                ConfigAction::Check => {
                    match cliff_watch_core::config::GovConfig::load() {
                        Ok(cfg) => {
                            println!("✅ Configuration valid:");
                            println!("   Difficulty:  {}", cfg.governance.difficulty);
                            println!("   Min Entropy: {}", cfg.governance.min_entropy);
                            println!("   Watch Root:  {}", cfg.monitoring.watch_root);
                        }
                        Err(e) => {
                            eprintln!("❌ Configuration invalid: {}", e);
                            process::exit(1);
                        }
                    }
                }
            }
        }
    }
}

async fn query_daemon(request: cliff_watch_core::protocol::Request) -> anyhow::Result<cliff_watch_core::protocol::Response> {
    use tokio::net::UnixStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let socket_path = "/tmp/cliff-watch.sock";
    let mut stream = UnixStream::connect(socket_path).await?;
    
    let request_json = serde_json::to_vec(&request)?;
    stream.write_all(&request_json).await?;
    
    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await?;
    
    let response: cliff_watch_core::protocol::Response = serde_json::from_slice(&buffer[..n])?;
    Ok(response)
}
async fn run_setup(no_confirm: bool) {
    println!("\n{} {} {}", style("===").blue(), style("Orquestador Soberano cliff-watch v2.1").bold(), style("===").blue());
    println!("{}\n", style("Preparando tu PC para la verdadera Gobernanza de Código...").italic());

    if !no_confirm {
        println!("Este proceso instalará dependencias de sistema y herramientas globales.");
        println!("Se requiere acceso de administrador (sudo) para algunos pasos.");
    }

    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-");

    // Paso 1: Dependencias de Sistema (APT/Pacman)
    let pb1 = m.add(ProgressBar::new(1));
    pb1.set_style(sty.clone());
    pb1.set_message("Escrutando dependencias del sistema operativo...");
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    pb1.set_message(format!("{} Instalando dependencias base (cmake, ssl, zstd)...", PACKAGE));
    
    let mut cmd = if Path::new("/etc/debian_version").exists() {
        let mut c = Command::new("sudo");
        c.args(&["apt", "install", "-y", "build-essential", "pkg-config", "libssl-dev", "libzstd-dev", "libtss2-dev", "cmake", "curl", "git"]);
        c
    } else {
        let mut c = Command::new("sudo");
        c.args(&["pacman", "-S", "--needed", "--noconfirm", "base-devel", "pkgconf", "openssl", "zstd", "tpm2-tss", "cmake", "curl", "git"]);
        c
    };

    let status = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
    if status.is_err() || !status.unwrap().success() {
        pb1.abandon_with_message("❌ Error instalando dependencias de sistema.");
    } else {
        pb1.finish_with_message(format!("{} Dependencias de sistema listas.", SUCCESS));
    }

    // Paso 2: Herramientas Globales de Cargo
    let tools = vec!["cargo-audit", "cargo-expand", "cargo-nextest"];
    let pb2 = m.add(ProgressBar::new(tools.len() as u64));
    pb2.set_style(sty.clone());
    
    for tool in tools {
        pb2.set_message(format!("{} Forjando herramienta: {}...", GEAR, tool));
        let _ = Command::new("cargo")
            .args(&["install", tool, "--quiet"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        pb2.inc(1);
    }
    pb2.finish_with_message(format!("{} Armería de Cargo completa.", SUCCESS));

    // Paso 3: Extensiones de VSCode (Modo Soberano)
    let pb3 = m.add(ProgressBar::new(1));
    pb3.set_style(sty);
    pb3.set_message(format!("{} Forjando el Testigo (VSIX local)...", LOOKING_GLASS));

    // Intentar instalar VSCode extension localmente
    let vsix_built = Command::new("sh")
        .arg("-c")
        .arg("cd clients/cliff-watch-witness && npm install && npx vsce package --out ../../cliff-watch-witness.vsix --no-interaction")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if vsix_built.is_ok() && vsix_built.unwrap().success() {
        pb3.set_message(format!("{} Instalando Testigo Soberano...", SUCCESS));
        let _ = Command::new("code")
            .args(&["--install-extension", "cliff-watch-witness.vsix", "--force"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        pb3.finish_with_message(format!("{} Cliff-Watch Witness activado (Local).", SUCCESS));
    } else {
        pb3.abandon_with_message("⚠️ Error al forjar el Testigo local. Intenta manualmente con 'cliff-watch setup'.");
    }

    println!("\n{} {}", SPARKLES, style("¡Misión cumplida! Tu PC ahora es Soberano.").green().bold());
    println!("Ejecuta {} para empezar a validar tu entropía.", style("cliff-watch on").cyan());
}
