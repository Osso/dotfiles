use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
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
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            if dry_run {
                println!("  Would create directory: {}", parent.display());
            } else {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
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

pub fn run_apply(config: &LinksConfig, source_dir: &Path, dry_run: bool) -> Result<()> {
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
    Ok(())
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
