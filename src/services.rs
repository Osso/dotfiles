use anyhow::Result;

use crate::config::SetupConfig;
use crate::utils::run_command;

pub fn run_services(config: &SetupConfig, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry run - no changes will be made\n");
    }

    // User services
    if !config.services.user.is_empty() {
        println!("User services:");
        for service in &config.services.user {
            if dry_run {
                println!("  Would enable: {}", service);
            } else {
                println!("  Enabling: {}", service);
                run_command("systemctl", &["--user", "enable", "--now", service], false)?;
            }
        }
    }

    // System services
    if !config.services.system.is_empty() {
        println!("\nSystem services:");
        for service in &config.services.system {
            if dry_run {
                println!("  Would enable: {}", service);
            } else {
                println!("  Enabling: {}", service);
                run_command("systemctl", &["enable", "--now", service], true)?;
            }
        }
    }

    // Create directories
    if !config.directories.is_empty() {
        println!("\nDirectories:");
        for dir in &config.directories {
            let path = crate::utils::expand_path(dir)?;
            if path.exists() {
                println!("  {} (exists)", dir);
            } else if dry_run {
                println!("  Would create: {}", dir);
            } else {
                std::fs::create_dir_all(&path)?;
                println!("  Created: {}", dir);
            }
        }
    }

    Ok(())
}
