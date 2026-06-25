#[cfg(not(coverage))]
use anyhow::Result;

#[cfg(not(coverage))]
use crate::config::SetupConfig;
#[cfg(not(coverage))]
use crate::utils::{expand_path, run_command};

#[cfg(not(coverage))]
pub fn run_services(config: &SetupConfig, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry run - no changes will be made\n");
    }

    run_service_group("User services", &config.services.user, dry_run, false)?;
    run_service_group("System services", &config.services.system, dry_run, true)?;
    create_directories(&config.directories, dry_run)?;
    Ok(())
}

#[cfg(not(coverage))]
fn run_service_group(
    title: &str,
    services: &[String],
    dry_run: bool,
    use_sudo: bool,
) -> Result<()> {
    if services.is_empty() {
        return Ok(());
    }

    println!("{title}:");
    for service in services {
        run_service(service, dry_run, use_sudo)?;
    }
    println!();
    Ok(())
}

#[cfg(not(coverage))]
fn run_service(service: &str, dry_run: bool, use_sudo: bool) -> Result<()> {
    if dry_run {
        println!("  Would enable: {}", service);
        return Ok(());
    }

    println!("  Enabling: {}", service);
    let args = service_args(service, use_sudo);
    run_command("systemctl", &args, use_sudo)
}

#[cfg(not(coverage))]
fn service_args(service: &str, system_service: bool) -> Vec<&str> {
    if system_service {
        vec!["enable", "--now", service]
    } else {
        vec!["--user", "enable", "--now", service]
    }
}

#[cfg(not(coverage))]
fn create_directories(directories: &[String], dry_run: bool) -> Result<()> {
    if directories.is_empty() {
        return Ok(());
    }

    println!("Directories:");
    for dir in directories {
        create_directory(dir, dry_run)?;
    }
    Ok(())
}

#[cfg(not(coverage))]
fn create_directory(dir: &str, dry_run: bool) -> Result<()> {
    let path = expand_path(dir)?;
    if path.exists() {
        println!("  {} (exists)", dir);
        return Ok(());
    }

    if dry_run {
        println!("  Would create: {}", dir);
        return Ok(());
    }

    std::fs::create_dir_all(&path)?;
    println!("  Created: {}", dir);
    Ok(())
}
