use anyhow::Result;
use clap::{Parser, Subcommand};

use ark::ArkPackageManager;
use ark::confirm::confirm;
use ark::types::{ArkCommand, ArkConfig};

#[derive(Parser)]
#[command(name = "ark", version = ark::ARK_VERSION, about = "Ark — unified package manager for AGNOS")]
struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install one or more packages
    Install {
        /// Packages to install
        packages: Vec<String>,
        /// Force reinstall
        #[arg(short, long)]
        force: bool,
        /// Install a package group instead
        #[arg(short, long)]
        group: Option<String>,
    },
    /// Remove one or more packages
    Remove {
        /// Packages to remove
        packages: Vec<String>,
        /// Purge configuration files too
        #[arg(long)]
        purge: bool,
    },
    /// Search for packages across all sources
    Search {
        /// Search query
        query: Vec<String>,
        /// Filter by source (system, marketplace, flutter)
        #[arg(short, long)]
        source: Option<String>,
    },
    /// List installed packages
    List {
        /// Show only marketplace packages
        #[arg(long)]
        marketplace: bool,
        /// Show only system packages
        #[arg(long)]
        system: bool,
        /// Show only flutter apps
        #[arg(long)]
        flutter: bool,
    },
    /// Show detailed info about a package
    Info {
        /// Package name
        package: String,
    },
    /// Check for updates across all sources
    Update,
    /// Upgrade packages with available updates
    Upgrade {
        /// Specific packages to upgrade (all if omitted)
        packages: Vec<String>,
    },
    /// Show ark version and system status
    Status,
    /// Hold packages to prevent upgrades
    Hold {
        /// Packages to hold
        packages: Vec<String>,
    },
    /// Remove hold on packages
    Unhold {
        /// Packages to unhold
        packages: Vec<String>,
    },
    /// Verify integrity of installed packages
    Verify {
        /// Package to verify (all if omitted)
        package: Option<String>,
    },
    /// Show transaction history
    History {
        /// Number of entries to show (default: 10)
        count: Option<usize>,
    },
}

fn to_ark_command(cmd: &Commands) -> Result<ArkCommand> {
    match cmd {
        Commands::Install {
            packages,
            force,
            group,
        } => {
            if let Some(g) = group {
                Ok(ArkCommand::GroupInstall {
                    group: g.clone(),
                    force: *force,
                })
            } else {
                if packages.is_empty() {
                    anyhow::bail!("install requires at least one package name or --group <name>");
                }
                Ok(ArkCommand::Install {
                    packages: packages.clone(),
                    force: *force,
                })
            }
        }
        Commands::Remove { packages, purge } => {
            if packages.is_empty() {
                anyhow::bail!("remove requires at least one package name");
            }
            Ok(ArkCommand::Remove {
                packages: packages.clone(),
                purge: *purge,
            })
        }
        Commands::Search { query, source } => {
            if query.is_empty() {
                anyhow::bail!("search requires a query");
            }
            let source = source
                .as_deref()
                .map(|s| match s.to_lowercase().as_str() {
                    "system" | "apt" => Ok(nous::PackageSource::System),
                    "marketplace" | "market" => Ok(nous::PackageSource::Marketplace),
                    "flutter" | "flutter-app" => Ok(nous::PackageSource::FlutterApp),
                    other => anyhow::bail!("Unknown source: '{}'", other),
                })
                .transpose()?;
            Ok(ArkCommand::Search {
                query: query.join(" "),
                source,
            })
        }
        Commands::List {
            marketplace,
            system,
            flutter,
        } => {
            let source = if *marketplace {
                Some(nous::PackageSource::Marketplace)
            } else if *system {
                Some(nous::PackageSource::System)
            } else if *flutter {
                Some(nous::PackageSource::FlutterApp)
            } else {
                None
            };
            Ok(ArkCommand::List { source })
        }
        Commands::Info { package } => Ok(ArkCommand::Info {
            package: package.clone(),
        }),
        Commands::Update => Ok(ArkCommand::Update),
        Commands::Upgrade { packages } => Ok(ArkCommand::Upgrade {
            packages: if packages.is_empty() {
                None
            } else {
                Some(packages.clone())
            },
        }),
        Commands::Status => Ok(ArkCommand::Status),
        Commands::Hold { packages } => {
            if packages.is_empty() {
                anyhow::bail!("hold requires at least one package name");
            }
            Ok(ArkCommand::Hold {
                packages: packages.clone(),
            })
        }
        Commands::Unhold { packages } => {
            if packages.is_empty() {
                anyhow::bail!("unhold requires at least one package name");
            }
            Ok(ArkCommand::Unhold {
                packages: packages.clone(),
            })
        }
        Commands::Verify { package } => Ok(ArkCommand::Verify {
            package: package.clone(),
        }),
        Commands::History { count } => Ok(ArkCommand::History { count: *count }),
    }
}

fn needs_confirmation(cmd: &ArkCommand, config: &ArkConfig) -> Option<String> {
    match cmd {
        ArkCommand::Install { packages, .. } => {
            if config.confirm_system_installs {
                Some(format!("Install {}?", packages.join(", ")))
            } else {
                None
            }
        }
        ArkCommand::GroupInstall { group, .. } => {
            if config.confirm_system_installs {
                Some(format!("Install group {}?", group))
            } else {
                None
            }
        }
        ArkCommand::Remove { packages, purge } => {
            if config.confirm_removals {
                let action = if *purge { "Purge" } else { "Remove" };
                Some(format!("{} {}?", action, packages.join(", ")))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let cli = Cli::parse();

    // Load config
    #[cfg(feature = "config")]
    let mut config = ark::config::load_config()?;
    #[cfg(not(feature = "config"))]
    let mut config = ArkConfig::default();

    if cli.no_color {
        config.color_output = false;
    }

    let ark_command = to_ark_command(&cli.command)?;

    // Confirmation prompt
    if let Some(prompt) = needs_confirmation(&ark_command, &config)
        && !confirm(&prompt)
    {
        println!("Cancelled.");
        return Ok(());
    }

    let mut manager = ArkPackageManager::new(config.clone())?;
    let result = manager.execute(&ark_command)?;

    // Build output and render
    let mut output = ark::ArkOutput::new();
    if result.success {
        output
            .lines
            .push(ark::ArkOutputLine::Success(result.message));
    } else {
        output.lines.push(ark::ArkOutputLine::Error(result.message));
    }

    print!("{}", output.render(config.color_output));

    if !result.success {
        std::process::exit(1);
    }

    Ok(())
}
