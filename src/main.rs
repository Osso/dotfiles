use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::{self, read_dir};
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "dotfiles", about = "Simple dotfiles symlink manager")]
struct Cli {
    /// Config file path
    #[arg(short, long, default_value = "~/.config/dotfiles/config.toml")]
    config: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show status of all symlinks
    Status,
    /// Create/update symlinks
    Apply {
        /// Don't actually create symlinks, just show what would happen
        #[arg(short = 'n', long)]
        dry_run: bool,
    },
    /// Verify all symlinks are correct
    Check,
}

#[derive(Deserialize)]
struct Config {
    source_dir: String,
    #[serde(default)]
    links: BTreeMap<String, String>,
    #[serde(default)]
    patterns: BTreeMap<String, String>,
    #[serde(default)]
    exclude: Vec<String>,
}

/// Expand patterns like "config/*" = "~/.config/*" into concrete links
fn expand_patterns(
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

        let src_prefix = &src_pattern[..src_pattern.len() - 1]; // remove trailing *
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

                // Skip excluded paths
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

#[derive(Debug, PartialEq)]
enum LinkStatus {
    Ok,
    Missing,
    WrongTarget { current: PathBuf },
    NotASymlink,
    BrokenSymlink,
}

fn expand_path(path: &str) -> Result<PathBuf> {
    if path.starts_with("~/") {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(&path[2..]))
    } else {
        Ok(PathBuf::from(path))
    }
}

fn get_link_status(source: &Path, dest: &Path) -> LinkStatus {
    let metadata = match dest.symlink_metadata() {
        Ok(m) => m,
        Err(_) => return LinkStatus::Missing,
    };

    if !metadata.file_type().is_symlink() {
        return LinkStatus::NotASymlink;
    }

    match dest.read_link() {
        Ok(target) => {
            // Canonicalize both paths to handle symlinks in the path
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

fn create_symlink(source: &Path, dest: &Path, dry_run: bool) -> Result<()> {
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

fn load_config(path: &str) -> Result<Config> {
    let path = expand_path(path)?;
    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;
    toml::from_str(&content).context("Failed to parse config")
}

fn run_status(config: &Config, source_dir: &Path) -> Result<()> {
    let mut ok = 0;
    let mut issues = 0;

    for (src, dest) in &config.links {
        let source = source_dir.join(src);
        let dest = expand_path(dest)?;
        let status = get_link_status(&source, &dest);

        match &status {
            LinkStatus::Ok => {
                println!("  \x1b[32m✓\x1b[0m {}", dest.display());
                ok += 1;
            }
            LinkStatus::Missing => {
                println!("  \x1b[33m○\x1b[0m {} (missing)", dest.display());
                issues += 1;
            }
            LinkStatus::WrongTarget { current } => {
                println!(
                    "  \x1b[31m✗\x1b[0m {} (points to {})",
                    dest.display(),
                    current.display()
                );
                issues += 1;
            }
            LinkStatus::NotASymlink => {
                println!("  \x1b[31m!\x1b[0m {} (not a symlink)", dest.display());
                issues += 1;
            }
            LinkStatus::BrokenSymlink => {
                println!("  \x1b[31m⚠\x1b[0m {} (broken symlink)", dest.display());
                issues += 1;
            }
        }
    }

    println!("\n{ok} ok, {issues} issues");
    Ok(())
}

fn run_apply(config: &Config, source_dir: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry run - no changes will be made\n");
    }

    let mut created = 0;
    let mut skipped = 0;

    for (src, dest) in &config.links {
        let source = source_dir.join(src);
        let dest = expand_path(dest)?;

        if !source.exists() {
            println!("  \x1b[33m!\x1b[0m Skipping {} (source missing)", src);
            skipped += 1;
            continue;
        }

        let status = get_link_status(&source, &dest);

        match status {
            LinkStatus::Ok => {
                skipped += 1;
            }
            LinkStatus::Missing | LinkStatus::WrongTarget { .. } | LinkStatus::BrokenSymlink => {
                create_symlink(&source, &dest, dry_run)?;
                created += 1;
            }
            LinkStatus::NotASymlink => {
                println!(
                    "  \x1b[31m!\x1b[0m Skipping {} (exists and not a symlink)",
                    dest.display()
                );
                skipped += 1;
            }
        }
    }

    println!("\n{created} created, {skipped} skipped");
    Ok(())
}

fn run_check(config: &Config, source_dir: &Path) -> Result<()> {
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = load_config(&cli.config)?;
    let source_dir = expand_path(&config.source_dir)?;

    if !source_dir.exists() {
        bail!("Source directory does not exist: {}", source_dir.display());
    }

    // Expand patterns and merge with explicit links
    let expanded = expand_patterns(&config.patterns, &config.exclude, &source_dir)?;
    for (src, dest) in expanded {
        // Explicit links take precedence over patterns
        config.links.entry(src).or_insert(dest);
    }

    match cli.command {
        Command::Status => run_status(&config, &source_dir),
        Command::Apply { dry_run } => run_apply(&config, &source_dir, dry_run),
        Command::Check => run_check(&config, &source_dir),
    }
}
