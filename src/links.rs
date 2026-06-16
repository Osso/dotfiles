use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, read_dir};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::config::LinksConfig;
use crate::utils::{color, expand_path};

#[derive(Debug, PartialEq)]
pub enum LinkStatus {
    Ok,
    Missing,
    WrongTarget { current: PathBuf },
    NotASymlink,
    BrokenSymlink,
}

/// Expand patterns like "config/*" = "~/.config/*" into concrete links
pub fn expand_patterns(
    patterns: &BTreeMap<String, String>,
    exclude: &[String],
    source_dir: &Path,
) -> Result<BTreeMap<String, String>> {
    let mut expanded = BTreeMap::new();

    for (src_pattern, dest_pattern) in patterns {
        if !src_pattern.ends_with("/*") || !dest_pattern.ends_with("/*") {
            bail!(
                "Pattern must end with /*: {} = {}",
                src_pattern,
                dest_pattern
            );
        }

        let src_prefix = &src_pattern[..src_pattern.len() - 1];
        let dest_prefix = &dest_pattern[..dest_pattern.len() - 1];
        let src_dir = source_dir.join(src_prefix);

        if !src_dir.exists() {
            continue;
        }

        for entry in read_dir(&src_dir)
            .with_context(|| format!("Failed to read directory: {}", src_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                let src = format!("{}{}", src_prefix, name_str);

                if exclude.iter().any(|e| e == &src) {
                    continue;
                }

                let dest = format!("{}{}", dest_prefix, name_str);
                expanded.insert(src, dest);
            }
        }
    }

    Ok(expanded)
}

pub fn get_link_status(source: &Path, dest: &Path) -> LinkStatus {
    let metadata = match dest.symlink_metadata() {
        Ok(m) => m,
        Err(_) => return LinkStatus::Missing,
    };

    if !metadata.file_type().is_symlink() {
        return LinkStatus::NotASymlink;
    }

    match dest.read_link() {
        Ok(target) => {
            let canonical_source = match source.canonicalize() {
                Ok(p) => p,
                Err(_) => return LinkStatus::BrokenSymlink,
            };
            let canonical_target = match target.canonicalize() {
                Ok(p) => p,
                Err(_) => return LinkStatus::BrokenSymlink,
            };

            if canonical_target == canonical_source {
                LinkStatus::Ok
            } else {
                LinkStatus::WrongTarget { current: target }
            }
        }
        Err(_) => LinkStatus::Missing,
    }
}

pub fn create_symlink(source: &Path, dest: &Path, dry_run: bool) -> Result<()> {
    if let Some(parent) = dest.parent()
        && !parent.exists()
    {
        if dry_run {
            println!("  Would create directory: {}", parent.display());
        } else {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }

    if dest.symlink_metadata().is_ok() {
        if dry_run {
            println!("  Would remove existing: {}", dest.display());
        } else {
            fs::remove_file(dest)
                .with_context(|| format!("Failed to remove existing file: {}", dest.display()))?;
        }
    }

    if dry_run {
        println!("  Would link: {} -> {}", dest.display(), source.display());
    } else {
        symlink(source, dest)
            .with_context(|| format!("Failed to create symlink: {}", dest.display()))?;
        println!("  Linked: {} -> {}", dest.display(), source.display());
    }

    Ok(())
}

pub fn run_status(config: &LinksConfig, source_dir: &Path) -> Result<()> {
    let mut ok = 0;
    let mut issues = 0;
    for (src, dest_pattern) in &config.links {
        let source = source_dir.join(src);
        let dest = expand_path(dest_pattern)?;
        let status = get_link_status(&source, &dest);
        print_link_status(&dest, &status);
        if matches!(status, LinkStatus::Ok) {
            ok += 1;
        } else {
            issues += 1;
        }
    }
    println!("\n{ok} ok, {issues} issues");
    Ok(())
}

pub fn run_apply(
    config: &LinksConfig,
    source_dir: &Path,
    dry_run: bool,
    prune: bool,
) -> Result<()> {
    if dry_run {
        println!("Dry run - no changes will be made\n");
    }

    let mut created = 0;
    let mut skipped = 0;

    for (src, dest) in &config.links {
        let source = source_dir.join(src);
        let dest = expand_path(dest)?;

        match apply_link(&source, &dest, src, dry_run)? {
            ApplyOutcome::Created => created += 1,
            ApplyOutcome::Skipped => skipped += 1,
        }
    }

    println!("\n{created} created, {skipped} skipped");

    if prune {
        let pruned = prune_orphans(config, source_dir, dry_run)?;
        let verb = if dry_run { "to prune" } else { "pruned" };
        println!("{pruned} orphaned link(s) {verb}");
    }

    Ok(())
}

/// Remove symlinks under managed destination dirs that point into `source_dir`
/// but are no longer declared — orphans left behind when a config is removed
/// from the source repo. Only ever removes symlinks whose target resolves
/// inside `source_dir`; never touches real files or foreign symlinks.
fn prune_orphans(config: &LinksConfig, source_dir: &Path, dry_run: bool) -> Result<usize> {
    let canon_source = source_dir
        .canonicalize()
        .unwrap_or_else(|_| source_dir.to_path_buf());

    // Declared destinations we intend to keep, and the dirs we scan for orphans.
    let mut declared: HashSet<PathBuf> = HashSet::new();
    let mut scan_dirs: HashSet<PathBuf> = HashSet::new();
    for dest in config.links.values() {
        let path = expand_path(dest)?;
        if let Some(parent) = path.parent() {
            scan_dirs.insert(parent.to_path_buf());
        }
        declared.insert(path);
    }

    let mut pruned = 0;
    for dir in &scan_dirs {
        let Ok(entries) = read_dir(dir) else { continue };
        for entry in entries {
            let path = entry?.path();
            if declared.contains(&path) || !points_into(&path, &canon_source) {
                continue;
            }
            if dry_run {
                println!("  Would prune orphan: {}", path.display());
            } else {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to prune: {}", path.display()))?;
                println!("  Pruned orphan: {}", path.display());
            }
            pruned += 1;
        }
    }
    Ok(pruned)
}

/// True only if `path` is a symlink whose target resolves inside `canon_source`.
fn points_into(path: &Path, canon_source: &Path) -> bool {
    let Ok(meta) = path.symlink_metadata() else {
        return false;
    };
    if !meta.file_type().is_symlink() {
        return false;
    }
    let Ok(raw) = path.read_link() else {
        return false;
    };
    let abs = if raw.is_absolute() {
        raw
    } else {
        path.parent().map(|p| p.join(&raw)).unwrap_or(raw)
    };
    match abs.canonicalize() {
        Ok(canonical) => canonical.starts_with(canon_source),
        Err(_) => abs.starts_with(canon_source),
    }
}

fn print_link_status(dest: &Path, status: &LinkStatus) {
    match status {
        LinkStatus::Ok => println!("  {}✓{} {}", color::GREEN, color::RESET, dest.display()),
        LinkStatus::Missing => println!(
            "  {}○{} {} (missing)",
            color::YELLOW,
            color::RESET,
            dest.display()
        ),
        LinkStatus::WrongTarget { current } => println!(
            "  {}✗{} {} (points to {})",
            color::RED,
            color::RESET,
            dest.display(),
            current.display()
        ),
        LinkStatus::NotASymlink => println!(
            "  {}!{} {} (not a symlink)",
            color::RED,
            color::RESET,
            dest.display()
        ),
        LinkStatus::BrokenSymlink => println!(
            "  {}⚠{} {} (broken symlink)",
            color::RED,
            color::RESET,
            dest.display()
        ),
    }
}

enum ApplyOutcome {
    Created,
    Skipped,
}

fn apply_link(source: &Path, dest: &Path, src_label: &str, dry_run: bool) -> Result<ApplyOutcome> {
    if !source.exists() {
        println!(
            "  {}!{} Skipping {} (source missing)",
            color::YELLOW,
            color::RESET,
            src_label
        );
        return Ok(ApplyOutcome::Skipped);
    }

    match get_link_status(source, dest) {
        LinkStatus::Ok => Ok(ApplyOutcome::Skipped),
        LinkStatus::Missing | LinkStatus::WrongTarget { .. } | LinkStatus::BrokenSymlink => {
            create_symlink(source, dest, dry_run)?;
            Ok(ApplyOutcome::Created)
        }
        LinkStatus::NotASymlink => {
            println!(
                "  {}!{} Skipping {} (exists and not a symlink)",
                color::RED,
                color::RESET,
                dest.display()
            );
            Ok(ApplyOutcome::Skipped)
        }
    }
}

pub fn run_check(config: &LinksConfig, source_dir: &Path) -> Result<()> {
    let mut all_ok = true;

    for (src, dest) in &config.links {
        let source = source_dir.join(src);
        let dest = expand_path(dest)?;
        let status = get_link_status(&source, &dest);

        if status != LinkStatus::Ok {
            all_ok = false;
            println!("{}: {:?}", dest.display(), status);
        }
    }

    if all_ok {
        println!("All symlinks OK");
        Ok(())
    } else {
        bail!("Some symlinks are incorrect")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn temp_dir(tag: &str) -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let p =
            std::env::temp_dir().join(format!("dotfiles-test-{tag}-{}-{n}", std::process::id()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    #[test]
    fn expand_patterns_lists_subdirs_and_honors_exclude() {
        let src = temp_dir("expand");
        for d in ["kitty", "git", "firefox"] {
            fs::create_dir_all(src.join("config").join(d)).unwrap();
        }
        // a file (not a dir) under config/ must be ignored
        fs::write(src.join("config/loose.txt"), "x").unwrap();

        let mut patterns = BTreeMap::new();
        patterns.insert("config/*".to_string(), "~/.config/*".to_string());
        let exclude = vec!["config/firefox".to_string()];

        let out = expand_patterns(&patterns, &exclude, &src).unwrap();

        assert_eq!(out.get("config/kitty").unwrap(), "~/.config/kitty");
        assert_eq!(out.get("config/git").unwrap(), "~/.config/git");
        assert!(!out.contains_key("config/firefox")); // excluded
        assert!(!out.contains_key("config/loose.txt")); // not a dir
        fs::remove_dir_all(&src).ok();
    }

    #[test]
    fn prune_removes_only_orphans_into_source() {
        let root = temp_dir("prune");
        let src = root.join("src");
        let home = root.join("home");
        fs::create_dir_all(src.join("keep")).unwrap();
        fs::create_dir_all(src.join("orphan")).unwrap();
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(root.join("elsewhere")).unwrap();

        // declared + correctly linked
        symlink(src.join("keep"), home.join("keep")).unwrap();
        // orphan: points INTO source but not declared -> must be pruned
        symlink(src.join("orphan"), home.join("orphan")).unwrap();
        // foreign: symlink NOT into source -> must be left alone
        symlink(root.join("elsewhere"), home.join("foreign")).unwrap();

        let mut links = BTreeMap::new();
        links.insert(
            "keep".to_string(),
            home.join("keep").to_string_lossy().to_string(),
        );
        let config = LinksConfig {
            source_dir: src.to_string_lossy().to_string(),
            links,
            patterns: BTreeMap::new(),
            exclude: vec![],
        };

        let pruned = prune_orphans(&config, &src, false).unwrap();
        assert_eq!(pruned, 1);
        assert!(
            home.join("keep").symlink_metadata().is_ok(),
            "declared link kept"
        );
        assert!(
            home.join("orphan").symlink_metadata().is_err(),
            "orphan pruned"
        );
        assert!(
            home.join("foreign").symlink_metadata().is_ok(),
            "foreign link untouched"
        );
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn prune_dry_run_removes_nothing() {
        let root = temp_dir("prune-dry");
        let src = root.join("src");
        let home = root.join("home");
        fs::create_dir_all(src.join("orphan")).unwrap();
        fs::create_dir_all(&home).unwrap();
        fs::create_dir_all(src.join("keep")).unwrap();
        symlink(src.join("keep"), home.join("keep")).unwrap();
        symlink(src.join("orphan"), home.join("orphan")).unwrap();

        let mut links = BTreeMap::new();
        links.insert(
            "keep".to_string(),
            home.join("keep").to_string_lossy().to_string(),
        );
        let config = LinksConfig {
            source_dir: src.to_string_lossy().to_string(),
            links,
            patterns: BTreeMap::new(),
            exclude: vec![],
        };

        let pruned = prune_orphans(&config, &src, true).unwrap();
        assert_eq!(pruned, 1); // counted
        assert!(
            home.join("orphan").symlink_metadata().is_ok(),
            "dry run kept orphan"
        );
        fs::remove_dir_all(&root).ok();
    }
}
