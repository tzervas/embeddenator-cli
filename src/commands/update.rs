//! Update command implementations (add, remove, modify, compact)
//!
//! Incremental update operations for engram files.

use anyhow::{Context, Result};
use embeddenator_fs::EmbrFS;
use embeddenator_io::{read_bincode_file, write_bincode_file};
use embeddenator_vsa::ReversibleVSAConfig;
use std::path::PathBuf;

/// Load an existing EmbrFS from engram and manifest files
fn load_embrfs(engram: &PathBuf, manifest: &PathBuf) -> Result<EmbrFS> {
    let engram_data = read_bincode_file(engram)
        .with_context(|| format!("Failed to read engram file: {}", engram.display()))?;
    let manifest_data = read_bincode_file(manifest)
        .with_context(|| format!("Failed to read manifest file: {}", manifest.display()))?;

    Ok(EmbrFS {
        engram: engram_data,
        manifest: manifest_data,
        resonator: None,
    })
}

/// Save EmbrFS to engram and manifest files
fn save_embrfs(fs: &EmbrFS, engram: &PathBuf, manifest: &PathBuf) -> Result<()> {
    write_bincode_file(engram, &fs.engram)
        .with_context(|| format!("Failed to write engram file: {}", engram.display()))?;
    write_bincode_file(manifest, &fs.manifest)
        .with_context(|| format!("Failed to write manifest file: {}", manifest.display()))?;
    Ok(())
}

pub fn handle_update_add(
    engram: PathBuf,
    manifest: PathBuf,
    file: PathBuf,
    logical_path: Option<String>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!(
            "Embeddenator v{} - Incremental Add",
            env!("CARGO_PKG_VERSION")
        );
        println!("===================================");
    }

    let mut fs = load_embrfs(&engram, &manifest)?;
    let config = ReversibleVSAConfig::default();

    let logical = logical_path.unwrap_or_else(|| {
        file.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string())
    });

    if verbose {
        println!("Adding file: {} -> {}", file.display(), logical);
    }

    fs.add_file(&file, logical.clone(), verbose, &config)
        .with_context(|| format!("Failed to add file: {}", file.display()))?;

    save_embrfs(&fs, &engram, &manifest)?;

    if verbose {
        println!("File added successfully: {}", logical);
    }

    Ok(())
}

pub fn handle_update_remove(
    engram: PathBuf,
    manifest: PathBuf,
    path: String,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!(
            "Embeddenator v{} - Incremental Remove",
            env!("CARGO_PKG_VERSION")
        );
        println!("======================================");
    }

    let mut fs = load_embrfs(&engram, &manifest)?;

    if verbose {
        println!("Removing file: {}", path);
    }

    fs.remove_file(&path, verbose)
        .with_context(|| format!("Failed to remove file: {}", path))?;

    save_embrfs(&fs, &engram, &manifest)?;

    if verbose {
        println!("File marked as deleted: {}", path);
        println!("Run 'compact' to permanently remove and reclaim space.");
    }

    Ok(())
}

pub fn handle_update_modify(
    engram: PathBuf,
    manifest: PathBuf,
    file: PathBuf,
    logical_path: Option<String>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!(
            "Embeddenator v{} - Incremental Modify",
            env!("CARGO_PKG_VERSION")
        );
        println!("======================================");
    }

    let mut fs = load_embrfs(&engram, &manifest)?;
    let config = ReversibleVSAConfig::default();

    let logical = logical_path.unwrap_or_else(|| {
        file.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string())
    });

    if verbose {
        println!("Modifying file: {} -> {}", file.display(), logical);
    }

    fs.modify_file(&file, logical.clone(), verbose, &config)
        .with_context(|| format!("Failed to modify file: {}", file.display()))?;

    save_embrfs(&fs, &engram, &manifest)?;

    if verbose {
        println!("File modified successfully: {}", logical);
    }

    Ok(())
}

pub fn handle_update_compact(engram: PathBuf, manifest: PathBuf, verbose: bool) -> Result<()> {
    if verbose {
        println!(
            "Embeddenator v{} - Compact Engram",
            env!("CARGO_PKG_VERSION")
        );
        println!("===================================");
    }

    let mut fs = load_embrfs(&engram, &manifest)?;
    let config = ReversibleVSAConfig::default();

    let deleted_before = fs.manifest.files.iter().filter(|f| f.deleted).count();

    if deleted_before == 0 {
        if verbose {
            println!("No deleted files to compact.");
        }
        return Ok(());
    }

    if verbose {
        println!("Compacting {} deleted files...", deleted_before);
    }

    fs.compact(verbose, &config)
        .context("Failed to compact engram")?;

    save_embrfs(&fs, &engram, &manifest)?;

    if verbose {
        println!(
            "Compaction complete. Removed {} deleted entries.",
            deleted_before
        );
    }

    Ok(())
}
