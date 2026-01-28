//! Update command implementations (add, remove, modify, compact)
//!
//! These commands enable incremental modifications to existing engrams
//! without full re-ingestion.

use anyhow::Result;
use embeddenator_fs::embrfs::EmbrFS;
use embeddenator_vsa::ReversibleVSAConfig;
use std::path::PathBuf;

/// Add a new file to an existing engram
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

    // Validate inputs
    if !engram.exists() {
        anyhow::bail!("Engram file not found: {}", engram.display());
    }
    if !manifest.exists() {
        anyhow::bail!("Manifest file not found: {}", manifest.display());
    }
    if !file.exists() {
        anyhow::bail!("Input file not found: {}", file.display());
    }

    // Load existing engram and manifest
    let engram_data = EmbrFS::load_engram(&engram)?;
    let manifest_data = EmbrFS::load_manifest(&manifest)?;

    // Create EmbrFS with loaded data
    let mut fs = EmbrFS {
        engram: engram_data,
        manifest: manifest_data,
        resonator: None,
    };
    let config = ReversibleVSAConfig::default();

    // Determine logical path
    let logical = logical_path.unwrap_or_else(|| {
        file.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string()
    });

    if verbose {
        println!("Adding file: {} -> {}", file.display(), logical);
    }

    // Add the file (will fail if already exists)
    fs.add_file(&file, logical.clone(), verbose, &config)?;

    // Save updated engram and manifest
    fs.save_engram(&engram)?;
    fs.save_manifest(&manifest)?;

    if verbose {
        println!("\nAdd complete!");
        println!("  File added: {}", logical);
        println!(
            "  Total files: {}",
            fs.manifest.files.iter().filter(|f| !f.deleted).count()
        );
    }

    Ok(())
}

/// Remove a file from the engram (mark as deleted)
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

    // Validate inputs
    if !engram.exists() {
        anyhow::bail!("Engram file not found: {}", engram.display());
    }
    if !manifest.exists() {
        anyhow::bail!("Manifest file not found: {}", manifest.display());
    }

    // Load existing engram and manifest
    let engram_data = EmbrFS::load_engram(&engram)?;
    let manifest_data = EmbrFS::load_manifest(&manifest)?;

    // Create EmbrFS with loaded data
    let mut fs = EmbrFS {
        engram: engram_data,
        manifest: manifest_data,
        resonator: None,
    };

    if verbose {
        println!("Removing file: {}", path);
    }

    // Remove the file (marks as deleted)
    fs.remove_file(&path, verbose)?;

    // Save updated manifest (engram doesn't change for removal)
    fs.save_manifest(&manifest)?;

    if verbose {
        println!("\nRemove complete!");
        println!("  File removed: {}", path);
        println!("  Note: Use 'update compact' to reclaim space from deleted files");
    }

    Ok(())
}

/// Modify an existing file in the engram
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

    // Validate inputs
    if !engram.exists() {
        anyhow::bail!("Engram file not found: {}", engram.display());
    }
    if !manifest.exists() {
        anyhow::bail!("Manifest file not found: {}", manifest.display());
    }
    if !file.exists() {
        anyhow::bail!("Input file not found: {}", file.display());
    }

    // Load existing engram and manifest
    let engram_data = EmbrFS::load_engram(&engram)?;
    let manifest_data = EmbrFS::load_manifest(&manifest)?;

    // Create EmbrFS with loaded data
    let mut fs = EmbrFS {
        engram: engram_data,
        manifest: manifest_data,
        resonator: None,
    };
    let config = ReversibleVSAConfig::default();

    // Determine logical path
    let logical = logical_path.unwrap_or_else(|| {
        file.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string()
    });

    if verbose {
        println!("Modifying file: {} -> {}", file.display(), logical);
    }

    // Modify the file (marks old as deleted, adds new version)
    fs.modify_file(&file, logical.clone(), verbose, &config)?;

    // Save updated engram and manifest
    fs.save_engram(&engram)?;
    fs.save_manifest(&manifest)?;

    if verbose {
        println!("\nModify complete!");
        println!("  File updated: {}", logical);
        println!("  Note: Use 'update compact' to reclaim space from old versions");
    }

    Ok(())
}

/// Compact the engram by rebuilding without deleted files
pub fn handle_update_compact(engram: PathBuf, manifest: PathBuf, verbose: bool) -> Result<()> {
    if verbose {
        println!(
            "Embeddenator v{} - Compact Engram",
            env!("CARGO_PKG_VERSION")
        );
        println!("===================================");
    }

    // Validate inputs
    if !engram.exists() {
        anyhow::bail!("Engram file not found: {}", engram.display());
    }
    if !manifest.exists() {
        anyhow::bail!("Manifest file not found: {}", manifest.display());
    }

    // Load existing engram and manifest
    let engram_data = EmbrFS::load_engram(&engram)?;
    let manifest_data = EmbrFS::load_manifest(&manifest)?;

    // Count deleted files before compact
    let deleted_count = manifest_data.files.iter().filter(|f| f.deleted).count();

    if deleted_count == 0 {
        if verbose {
            println!("No deleted files to compact. Engram is already optimal.");
        }
        return Ok(());
    }

    // Create EmbrFS with loaded data
    let mut fs = EmbrFS {
        engram: engram_data,
        manifest: manifest_data,
        resonator: None,
    };
    let config = ReversibleVSAConfig::default();

    if verbose {
        println!("Found {} deleted file(s) to remove", deleted_count);
        println!("Rebuilding engram...");
    }

    // Compact (rebuilds without deleted files)
    fs.compact(verbose, &config)?;

    // Save compacted engram and manifest
    fs.save_engram(&engram)?;
    fs.save_manifest(&manifest)?;

    if verbose {
        println!("\nCompact complete!");
        println!("  Removed {} deleted file(s)", deleted_count);
        println!(
            "  Active files: {}",
            fs.manifest.files.iter().filter(|f| !f.deleted).count()
        );
    }

    Ok(())
}
