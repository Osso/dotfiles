#[cfg(not(coverage))]
use anyhow::{Context, Result};
#[cfg(not(coverage))]
use std::process::Command;

#[cfg(not(coverage))]
use crate::utils::run_command;

/// Subvolume that holds snapshots, the mountpoint we use, and the prefix that
/// marks *our* generation snapshots — distinct from manual `@arch-*` snapshots
/// so auto-prune never touches anything we didn't create.
const SNAP_SUBVOL: &str = "@snapshots";
#[cfg(not(coverage))]
const MOUNT: &str = "/run/dotfiles/snapshots";
const PREFIX: &str = "gen-";

/// Take a read-only snapshot of `/` (the @arch root) as a generation, then
/// prune to the newest `keep`. A no-op when `keep` is 0 (feature disabled).
#[cfg(not(coverage))]
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

#[cfg(not(coverage))]
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

#[cfg(not(coverage))]
fn timestamp() -> Result<String> {
    let out = Command::new("date")
        .arg("+%Y%m%d-%H%M%S")
        .output()
        .context("Failed to run date")?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Backing device of `/` (e.g. `/dev/nvme0n1p8`), stripped of the `[subvol]` tag.
#[cfg(not(coverage))]
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
#[cfg(not(coverage))]
fn list_generations() -> Result<Vec<String>> {
    let out = Command::new("authsudo")
        .args(["btrfs", "subvolume", "list", "/"])
        .output()
        .context("Failed to list subvolumes")?;
    Ok(parse_generation_names(&String::from_utf8_lossy(
        &out.stdout,
    )))
}

/// Extract our generation names (gen-*) from `btrfs subvolume list` output,
/// sorted oldest first. Excludes manual `@arch-*`/`@home-*` snapshots — the
/// prefix filter is the safety boundary that keeps auto-prune from touching them.
fn parse_generation_names(list_output: &str) -> Vec<String> {
    let prefix = format!("{SNAP_SUBVOL}/");
    let mut gens: Vec<String> = list_output
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .filter_map(|path| path.strip_prefix(prefix.as_str()))
        .filter(|name| name.starts_with(PREFIX))
        .map(String::from)
        .collect();
    gens.sort();
    gens
}

/// Generations beyond the newest `keep` (the ones to delete).
fn prunable(gens: &[String], keep: u32) -> &[String] {
    let keep = keep as usize;
    let cut = gens.len().saturating_sub(keep);
    &gens[..cut]
}

#[cfg(not(coverage))]
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
#[cfg(not(coverage))]
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
ID 256 gen 1 top level 5 path @arch
ID 258 gen 1 top level 5 path @snapshots
ID 263 gen 1 top level 258 path @snapshots/@arch-2025-12-24-clean
ID 264 gen 1 top level 258 path @snapshots/@home-2025-12-24-clean
ID 290 gen 1 top level 258 path @snapshots/gen-20260616-062601
ID 288 gen 1 top level 258 path @snapshots/gen-20260616-062548
ID 289 gen 1 top level 258 path @snapshots/gen-20260616-062554";

    #[test]
    fn parse_keeps_only_gen_prefixed_sorted() {
        let gens = parse_generation_names(SAMPLE);
        assert_eq!(
            gens,
            vec![
                "gen-20260616-062548",
                "gen-20260616-062554",
                "gen-20260616-062601",
            ]
        );
    }

    #[test]
    fn parse_excludes_manual_snapshots() {
        // the @arch-*/@home-* manual snapshots must never appear (prune safety)
        let gens = parse_generation_names(SAMPLE);
        assert!(gens.iter().all(|g| g.starts_with("gen-")));
    }

    #[test]
    fn prunable_respects_keep() {
        let g: Vec<String> = (0..6).map(|i| format!("gen-{i}")).collect();
        assert_eq!(prunable(&g, 5), &g[..1]); // oldest one over the cap
        assert_eq!(prunable(&g, 6).len(), 0);
        assert_eq!(prunable(&g, 10).len(), 0); // keep > count
        assert_eq!(prunable(&g, 0), &g[..]); // keep 0 → all prunable
    }
}
