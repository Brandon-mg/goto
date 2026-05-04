use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::Select;
use std::path::PathBuf;
use tracing::debug;
use walkdir::WalkDir;

mod config;

use config::{load_config, load_depth, save_config, save_depth, DEFAULT_DEPTH};

#[derive(Parser, Debug)]
#[command(name = "goto", about = "Jump to directories by fuzzy alias")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Jump to a directory
    #[command(visible_alias = "j")]
    Jump {
        #[arg(long, short = 'e')]
        exact: bool,
        #[arg(value_name = "TARGET")]
        target: String,
        #[arg(value_name = "ROOT")]
        root: Option<String>,
    },

    /// Add a namespace directory
    #[command(visible_alias = "a")]
    Add {
        name: String,
        path: PathBuf,
    },

    /// Show or update settings
    #[command(visible_alias = "s")]
    Settings {
        #[arg(long, short = 'd')]
        depth: Option<usize>,
        #[arg(long, short = 'r')]
        reset: bool,
    },

    /// List configured namespaces and settings
    #[command(visible_alias = "l")]
    List,
}

fn prompt_select(matches: &[PathBuf]) -> Result<usize> {
    let items: Vec<String> = matches.iter().map(|p| p.display().to_string()).collect();
    Select::new()
        .with_prompt("Multiple matches found")
        .items(&items)
        .default(0)
        .interact_opt()
        .context("Failed to show selection menu")?
        .context("No selection made")
}

fn resolve_target(target: &str, root: &std::path::Path, exact: bool) -> Result<PathBuf> {
    let target_lower = target.to_lowercase();
    let max_depth = load_depth();
    debug!("resolving target '{}' in {} (exact={}, depth={})", target_lower, root.display(), exact, max_depth);

    let matches: Vec<PathBuf> = WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .filter_map(|entry| {
            let name_lower = entry.file_name().to_str()?.to_lowercase();
            // short targets or exact mode require literal match;
            // otherwise allow a single typo via levenshtein distance
            let matches = if exact || target_lower.len() < 3 {
                target_lower == name_lower
            } else {
                strsim::levenshtein(&target_lower, &name_lower) <= 1
            };
            if matches {
                debug!("match: '{}' -> {}", name_lower, entry.path().display());
                Some(entry.path().to_path_buf())
            } else {
                None
            }
        })
        .collect();

    debug!("found {} matches for target '{}'", matches.len(), target_lower);

    if matches.is_empty() {
        anyhow::bail!("No matching directory found for '{}'", target_lower)
    }

    if matches.len() == 1 {
        return Ok(matches[0].clone());
    }

    prompt_select(&matches).map(|i| matches[i].clone())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let mut cfg = load_config()?;
    let depth = load_depth();

    match cli.command {
        Command::Jump { exact, target, root } => {
            debug!("jump command: target='{}' root={:?} exact={}", target, root, exact);

            let root = match root {
                Some(ns_or_path) => {
                    if let Some(path) = cfg.dirs.get(&ns_or_path) {
                        path.clone()
                    } else {
                        let p = PathBuf::from(&ns_or_path);
                        p.canonicalize().with_context(|| format!("Not a valid namespace or path: {}", ns_or_path))?
                    }
                }
                None => dirs::home_dir().context("No home directory")?,
            };

            let resolved = resolve_target(&target, &root, exact)?;
            println!("{}", resolved.display());
        }

        Command::Add { name, path } => {
            debug!("add command: name='{}' path={}", name, path.display());
            let canonical = path.canonicalize().unwrap_or(path);
            anyhow::ensure!(
                canonical.is_dir(),
                "Path is not a directory: {}",
                canonical.display()
            );
            cfg.dirs.insert(name.clone(), canonical.clone());
            save_config(&cfg)?;
            println!("Added {} -> {}", name, canonical.display());
        }

        Command::Settings { depth: new_depth, reset } => {
            debug!("settings command: depth={:?} reset={}", new_depth, reset);
            if reset {
                let path = config::depth_path();
                if path.exists() {
                    std::fs::remove_file(&path)?;
                    debug!("removed depth file at {:?}", path);
                }
                println!("Reset depth to default ({})", DEFAULT_DEPTH);
            } else if let Some(d) = new_depth {
                save_depth(d)?;
                println!("Set depth = {}", d);
            } else {
                println!("depth = {}", depth);
            }
        }

        Command::List => {
            debug!("list command");
            println!("home -> {}", dirs::home_dir().unwrap_or_default().display());
            for (name, path) in &cfg.dirs {
                println!("{} -> {}", name, path.display());
            }
            println!("\nsettings:\n  depth = {}", depth);
        }
    }

    Ok(())
}
