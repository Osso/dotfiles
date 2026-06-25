#[cfg(not(coverage))]
use anyhow::Result;
#[cfg(not(coverage))]
use std::path::{Path, PathBuf};

#[cfg(not(coverage))]
use crate::config::SetupConfig;
#[cfg(not(coverage))]
use crate::utils::run_command;

/// Point /etc/localtime at the declared zoneinfo file. Uses a symlink rather
/// than `timedatectl` so it also works inside a bootstrap chroot.
#[cfg(not(coverage))]
pub fn run_timezone(config: &SetupConfig, dry_run: bool) -> Result<()> {
    let Some(tz) = &config.timezone else {
        return Ok(());
    };

    let target = PathBuf::from(format!("/usr/share/zoneinfo/{tz}"));
    if Path::new("/etc/localtime").read_link().ok().as_ref() == Some(&target) {
        println!("Timezone: {tz} (ok)");
        return Ok(());
    }

    if dry_run {
        println!("Timezone: would set to {tz}");
        return Ok(());
    }

    println!("Timezone: setting {tz}");
    run_command(
        "ln",
        &["-sf", &target.to_string_lossy(), "/etc/localtime"],
        true,
    )
}
