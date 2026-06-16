use anyhow::{Context, Result};
use std::process::Command;

use crate::utils::run_command;

/// Subvolume that holds snapshots, the mountpoint we use, and the prefix that
/// marks *our* generation snapshots — distinct from manual `@arch-*` snapshots
/// so auto-prune never touches anything we didn't create.
const SNAP_SUBVOL: &str = "@snapshots";
const MOUNT: &str = "/run/dotfiles/snapshots";
const PREFIX: &str = "gen-";

/// Take a read-only snapshot of `/` (the @arch root) as a generation, then
/// prune to the newest `keep`. A no-op when `keep` is 0 (feature disabled).
pub fn run_snapshot(keep: u32, dry_run: bool) -> Result<()> {
    if keep == 0 {
        return Ok(());
    }

    let name = format!("{PREFIX}{}", timestamp()?);
    if dry_run {
        println!("Generations: would snapshot / -> {SNAP_SUBVOL}/{name} (keep {keep})");
        let existing = list_generations()?;
        for stale in prunable(&existing, keep) {
            println!("  would prune old generation: {stale}");
        }
        return Ok(());
    }

    with_snapshots_mounted(|| {
        println!("Generations: snapshotting / -> {name}");
        run_command(
            "btrfs",
            &[
                "subvolume",
                "snapshot",
                "-r",
                "/",
                &format!("{MOUNT}/{name}"),
            ],
            true,
        )?;
        prune(keep)
    })
}

pub fn run_list() -> Result<()> {
    let gens = list_generations()?;
    if gens.is_empty() {
        println!("No generations.");
        return Ok(());
    }
    println!("Generations (oldest first):");
    for g in &gens {
        println!("  {g}");
    }
    println!(
        "\nRoll back (needs reboot): mount -o subvol={SNAP_SUBVOL} <dev> /mnt, then swap\n\
         @arch with the chosen gen-* snapshot (rename @arch -> @arch.bad; btrfs subvolume\n\
         snapshot /mnt/<gen> /mnt/@arch) and reboot."
    );
    Ok(())
}

fn timestamp() -> Result<String> {
    let out = Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output()
        .context("Failed to run date")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Backing device of `/` (e.g. `/dev/nvme0n1p8`), stripped of the `[subvol]` tag.
fn device() -> Result<String> {
    let out = Command::new("findmnt")
        .args(["-no", "SOURCE", "/"])
        .output()
        .context("Failed to run findmnt")?;
    let raw = String::from_utf8_lossy(&out.stdout);
    Ok(raw.trim().split('[').next().unwrap_or("").to_string())
}

/// Our generation snapshot names (gen-*), sorted oldest first. The timestamp
/// format sorts lexically, so name order == chronological order.
fn list_generations() -> Result<Vec<String>> {
    let out = Command::new("authsudo")
        .args(["btrfs", "subvolume", "list", "/"])
        .output()
        .context("Failed to list subvolumes")?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut gens: Vec<String> = text
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .filter_map(|path| path.strip_prefix(&format!("{SNAP_SUBVOL}/")))
        .filter(|name| name.starts_with(PREFIX))
        .map(String::from)
        .collect();
    gens.sort();
    Ok(gens)
}

/// Generations beyond the newest `keep` (the ones to delete).
fn prunable(gens: &[String], keep: u32) -> &[String] {
    let keep = keep as usize;
    let cut = gens.len().saturating_sub(keep);
    &gens[..cut]
}

fn prune(keep: u32) -> Result<()> {
    let gens = list_generations()?;
    for name in prunable(&gens, keep) {
        println!("  Pruning old generation: {name}");
        run_command(
            "btrfs",
            &["subvolume", "delete", &format!("{MOUNT}/{name}")],
            true,
        )?;
    }
    Ok(())
}

/// Mount @snapshots at a scratch mountpoint, run `f`, always unmount.
fn with_snapshots_mounted<F: FnOnce() -> Result<()>>(f: F) -> Result<()> {
    let dev = device()?;
    run_command("mkdir", &["-p", MOUNT], true)?;
    run_command(
        "mount",
        &["-o", &format!("subvol={SNAP_SUBVOL}"), &dev, MOUNT],
        true,
    )?;
    let result = f();
    let _ = run_command("umount", &[MOUNT], true);
    result
}
