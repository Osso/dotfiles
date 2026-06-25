mod config;
mod generations;
mod links;
mod modules;
mod services;
mod timezone;
mod users;
mod utils;

#[cfg(not(coverage))]
use anyhow::{Result, bail};
#[cfg(not(coverage))]
use clap::{Parser, Subcommand};

#[cfg(not(coverage))]
use config::{LinksConfig, SetupConfig};
#[cfg(not(coverage))]
use links::expand_patterns;
#[cfg(not(coverage))]
use utils::expand_path;

#[cfg(not(coverage))]
#[derive(Parser)]
#[command(name = "dotfiles", about = "Dotfiles and system configuration manager")]
struct Cli {
    /// Links config file path
    #[arg(short, long, default_value = "~/.config/dotfiles/config.toml")]
    config: String,

    #[command(subcommand)]
    command: Command,
}

#[cfg(not(coverage))]
#[derive(Subcommand)]
enum Command {
    /// Show status of all symlinks
    Status,
    /// Create/update symlinks
    Apply {
        /// Don't actually make changes, just show what would happen
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// Also remove orphaned symlinks no longer in the source repo
        #[arg(long)]
        prune: bool,
    },
    /// Verify all symlinks are correct
    Check,
    /// Apply system modules (sysctl, fonts, etc.)
    System {
        #[command(subcommand)]
        command: Option<SystemCommand>,
        /// Don't actually make changes
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// Also remove /etc files we placed before but no longer produce
        #[arg(long)]
        prune: bool,
    },
    /// Enable/disable systemd services
    Services {
        /// Don't actually make changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// Ensure declared users exist with the right shell and groups
    Users {
        /// Don't actually make changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// Take a pre-apply root snapshot ("generation"), pruning to the configured keep
    Snapshot {
        /// Don't actually make changes
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// List generation snapshots
    Generations,
    /// Run full setup: apply + system + services
    Setup {
        /// Don't actually make changes
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// Also remove orphaned symlinks no longer in the source repo
        #[arg(long)]
        prune: bool,
    },
}

#[cfg(not(coverage))]
#[derive(Subcommand)]
enum SystemCommand {
    /// Show status of system modules
    Status,
}

#[cfg(not(coverage))]
fn main() -> Result<()> {
    let cli = Cli::parse();
    dispatch_command(&cli)
}

#[cfg(not(coverage))]
fn dispatch_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Status | Command::Apply { .. } | Command::Check => {
            run_links_command(cli)?;
        }
        Command::System {
            command,
            dry_run,
            prune,
        } => run_system_command(cli, command.as_ref(), *dry_run, *prune)?,
        Command::Services { dry_run } => {
            let setup_config = load_setup_config(&cli.config)?;
            services::run_services(&setup_config, *dry_run)?;
        }
        Command::Users { dry_run } => {
            let setup_config = load_setup_config(&cli.config)?;
            users::run_users(&setup_config, *dry_run)?;
        }
        Command::Snapshot { dry_run } => {
            let setup_config = load_setup_config(&cli.config)?;
            generations::run_snapshot(setup_config.generations, *dry_run)?;
        }
        Command::Generations => {
            generations::run_list()?;
        }
        Command::Setup { dry_run, prune } => {
            run_setup_command(cli, *dry_run, *prune)?;
        }
    };
    Ok(())
}

#[cfg(not(coverage))]
fn get_source_dir(config_path: &str) -> Result<std::path::PathBuf> {
    let config = LinksConfig::load(config_path)?;
    expand_path(&config.source_dir)
}

#[cfg(not(coverage))]
fn load_setup_config(config_path: &str) -> Result<SetupConfig> {
    let source_dir = get_source_dir(config_path)?;
    SetupConfig::load(&source_dir.join("setup.yaml"))
}

#[cfg(not(coverage))]
fn run_system_command(
    cli: &Cli,
    command: Option<&SystemCommand>,
    dry_run: bool,
    prune: bool,
) -> Result<()> {
    let source_dir = get_source_dir(&cli.config)?;
    let setup_config = SetupConfig::load(&source_dir.join("setup.yaml"))?;

    match command {
        Some(SystemCommand::Status) => modules::run_status(&setup_config, &source_dir),
        None => {
            modules::run_apply(&setup_config, &source_dir, dry_run, prune)?;
            timezone::run_timezone(&setup_config, dry_run)
        }
    }
}

#[cfg(not(coverage))]
fn run_setup_command(cli: &Cli, dry_run: bool, prune: bool) -> Result<()> {
    let source_dir = get_source_dir(&cli.config)?;
    let setup_config = SetupConfig::load(&source_dir.join("setup.yaml"))?;

    println!("=== Taking generation snapshot ===");
    generations::run_snapshot(setup_config.generations, dry_run)?;

    println!("\n=== Ensuring users ===");
    users::run_users(&setup_config, dry_run)?;

    println!("\n=== Applying symlinks ===");
    run_links_command_with_dry_run(cli, dry_run, prune)?;

    println!("\n=== Applying system modules ===");
    modules::run_apply(&setup_config, &source_dir, dry_run, prune)?;
    timezone::run_timezone(&setup_config, dry_run)?;

    println!("\n=== Enabling services ===");
    services::run_services(&setup_config, dry_run)?;

    println!("\n=== Setup complete ===");
    Ok(())
}

#[cfg(not(coverage))]
fn run_links_command(cli: &Cli) -> Result<()> {
    let mut config = LinksConfig::load(&cli.config)?;
    let source_dir = expand_path(&config.source_dir)?;

    if !source_dir.exists() {
        bail!("Source directory does not exist: {}", source_dir.display());
    }

    // Expand patterns and merge with explicit links
    let expanded = expand_patterns(&config.patterns, &config.exclude, &source_dir)?;
    for (src, dest) in expanded {
        config.links.entry(src).or_insert(dest);
    }

    match &cli.command {
        Command::Status => links::run_status(&config, &source_dir),
        Command::Apply { dry_run, prune } => {
            links::run_apply(&config, &source_dir, *dry_run, *prune)
        }
        Command::Check => links::run_check(&config, &source_dir),
        _ => Ok(()),
    }
}

#[cfg(not(coverage))]
fn run_links_command_with_dry_run(cli: &Cli, dry_run: bool, prune: bool) -> Result<()> {
    let mut config = LinksConfig::load(&cli.config)?;
    let source_dir = expand_path(&config.source_dir)?;

    if !source_dir.exists() {
        bail!("Source directory does not exist: {}", source_dir.display());
    }

    let expanded = expand_patterns(&config.patterns, &config.exclude, &source_dir)?;
    for (src, dest) in expanded {
        config.links.entry(src).or_insert(dest);
    }

    links::run_apply(&config, &source_dir, dry_run, prune)
}
