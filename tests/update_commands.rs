//! E2E tests for update commands (add, remove, modify, compact)

use embeddenator_cli::commands;
use std::fs;
use tempfile::TempDir;

fn setup_test_engram() -> (TempDir, std::path::PathBuf, std::path::PathBuf) {
    use embeddenator_fs::embrfs::EmbrFS;
    use embeddenator_vsa::ReversibleVSAConfig;

    let temp_dir = TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("input");
    fs::create_dir(&input_dir).unwrap();

    // Create initial files
    fs::write(input_dir.join("file1.txt"), b"Hello, World!").unwrap();
    fs::write(input_dir.join("file2.txt"), b"Test content for file 2").unwrap();

    // Ingest the files (use holographic mode for better storage efficiency)
    let mut embrfs = EmbrFS::new_holographic();
    let config = ReversibleVSAConfig::default();
    embrfs.ingest_directory(&input_dir, false, &config).unwrap();

    let engram_path = temp_dir.path().join("test.engram");
    let manifest_path = temp_dir.path().join("test.json");

    embrfs.save_engram(&engram_path).unwrap();
    embrfs.save_manifest(&manifest_path).unwrap();

    (temp_dir, engram_path, manifest_path)
}

#[test]
fn test_update_add() {
    let (temp_dir, engram_path, manifest_path) = setup_test_engram();

    // Create a new file to add
    let new_file = temp_dir.path().join("new_file.txt");
    fs::write(&new_file, b"New file content").unwrap();

    // Add the file
    let result = commands::handle_update_add(
        engram_path.clone(),
        manifest_path.clone(),
        new_file,
        Some("new_file.txt".to_string()),
        false,
    );

    assert!(result.is_ok(), "Add should succeed: {:?}", result);

    // Verify the file was added by loading the manifest
    let manifest = embeddenator_fs::EmbrFS::load_manifest(&manifest_path).unwrap();
    let found = manifest
        .files
        .iter()
        .any(|f| f.path == "new_file.txt" && !f.deleted);
    assert!(found, "New file should be in manifest");
}

#[test]
fn test_update_add_duplicate_fails() {
    let (temp_dir, engram_path, manifest_path) = setup_test_engram();

    // Try to add a file that already exists
    let dup_file = temp_dir.path().join("dup.txt");
    fs::write(&dup_file, b"Duplicate content").unwrap();

    // First add should succeed
    let result1 = commands::handle_update_add(
        engram_path.clone(),
        manifest_path.clone(),
        dup_file.clone(),
        Some("file1.txt".to_string()), // Already exists
        false,
    );
    assert!(
        result1.is_err(),
        "Adding duplicate file should fail: {:?}",
        result1
    );
}

#[test]
fn test_update_remove() {
    let (_temp_dir, engram_path, manifest_path) = setup_test_engram();

    // Remove a file
    let result = commands::handle_update_remove(
        engram_path.clone(),
        manifest_path.clone(),
        "file1.txt".to_string(),
        false,
    );

    assert!(result.is_ok(), "Remove should succeed: {:?}", result);

    // Verify the file was marked as deleted
    let manifest = embeddenator_fs::EmbrFS::load_manifest(&manifest_path).unwrap();
    let file = manifest.files.iter().find(|f| f.path == "file1.txt");
    assert!(file.is_some(), "File should still be in manifest");
    assert!(file.unwrap().deleted, "File should be marked as deleted");
}

#[test]
fn test_update_remove_nonexistent_fails() {
    let (_temp_dir, engram_path, manifest_path) = setup_test_engram();

    // Try to remove a file that doesn't exist
    let result = commands::handle_update_remove(
        engram_path,
        manifest_path,
        "nonexistent.txt".to_string(),
        false,
    );

    assert!(
        result.is_err(),
        "Removing nonexistent file should fail: {:?}",
        result
    );
}

#[test]
fn test_update_modify() {
    let (temp_dir, engram_path, manifest_path) = setup_test_engram();

    // Create a file with updated content
    let updated_file = temp_dir.path().join("updated.txt");
    fs::write(&updated_file, b"Updated content for file1").unwrap();

    // Modify file1.txt
    let result = commands::handle_update_modify(
        engram_path.clone(),
        manifest_path.clone(),
        updated_file,
        Some("file1.txt".to_string()),
        false,
    );

    assert!(result.is_ok(), "Modify should succeed: {:?}", result);

    // Verify the modification
    let manifest = embeddenator_fs::EmbrFS::load_manifest(&manifest_path).unwrap();

    // Should have one deleted file1.txt and one active file1.txt
    let file1_entries: Vec<_> = manifest
        .files
        .iter()
        .filter(|f| f.path == "file1.txt")
        .collect();
    assert_eq!(file1_entries.len(), 2, "Should have old and new file1.txt");

    let deleted_count = file1_entries.iter().filter(|f| f.deleted).count();
    let active_count = file1_entries.iter().filter(|f| !f.deleted).count();
    assert_eq!(deleted_count, 1, "Old version should be deleted");
    assert_eq!(active_count, 1, "New version should be active");
}

#[test]
fn test_update_compact() {
    let (_temp_dir, engram_path, manifest_path) = setup_test_engram();

    // First, remove a file
    commands::handle_update_remove(
        engram_path.clone(),
        manifest_path.clone(),
        "file1.txt".to_string(),
        false,
    )
    .unwrap();

    // Verify we have a deleted file
    let manifest_before = embeddenator_fs::EmbrFS::load_manifest(&manifest_path).unwrap();
    let deleted_before = manifest_before.files.iter().filter(|f| f.deleted).count();
    assert_eq!(
        deleted_before, 1,
        "Should have 1 deleted file before compact"
    );

    // Compact
    let result = commands::handle_update_compact(engram_path.clone(), manifest_path.clone(), false);

    assert!(result.is_ok(), "Compact should succeed: {:?}", result);

    // Verify deleted files are gone
    let manifest_after = embeddenator_fs::EmbrFS::load_manifest(&manifest_path).unwrap();
    let deleted_after = manifest_after.files.iter().filter(|f| f.deleted).count();
    assert_eq!(
        deleted_after, 0,
        "Should have no deleted files after compact"
    );

    // Remaining file should still be there
    let active_files: Vec<_> = manifest_after.files.iter().filter(|f| !f.deleted).collect();
    assert_eq!(active_files.len(), 1, "Should have 1 active file");
    assert_eq!(active_files[0].path, "file2.txt");
}

#[test]
fn test_update_compact_no_deleted_files() {
    let (_temp_dir, engram_path, manifest_path) = setup_test_engram();

    // Compact when there are no deleted files should succeed (early return)
    let result = commands::handle_update_compact(engram_path, manifest_path, false);

    assert!(
        result.is_ok(),
        "Compact with no deleted files should succeed: {:?}",
        result
    );
}

#[test]
fn test_update_full_workflow() {
    let (temp_dir, engram_path, manifest_path) = setup_test_engram();

    // 1. Add a new file
    let new_file = temp_dir.path().join("new.txt");
    fs::write(&new_file, b"New content").unwrap();
    commands::handle_update_add(
        engram_path.clone(),
        manifest_path.clone(),
        new_file,
        Some("new.txt".to_string()),
        false,
    )
    .unwrap();

    // 2. Modify an existing file
    let modified = temp_dir.path().join("modified.txt");
    fs::write(&modified, b"Modified content").unwrap();
    commands::handle_update_modify(
        engram_path.clone(),
        manifest_path.clone(),
        modified,
        Some("file1.txt".to_string()),
        false,
    )
    .unwrap();

    // 3. Remove a file
    commands::handle_update_remove(
        engram_path.clone(),
        manifest_path.clone(),
        "file2.txt".to_string(),
        false,
    )
    .unwrap();

    // 4. Compact
    commands::handle_update_compact(engram_path.clone(), manifest_path.clone(), false).unwrap();

    // Verify final state
    let manifest = embeddenator_fs::EmbrFS::load_manifest(&manifest_path).unwrap();
    let active: Vec<_> = manifest.files.iter().filter(|f| !f.deleted).collect();
    assert_eq!(active.len(), 2, "Should have 2 active files");

    let paths: Vec<_> = active.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"new.txt"), "Should have new.txt");
    assert!(
        paths.contains(&"file1.txt"),
        "Should have modified file1.txt"
    );

    // Verify extraction works
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&output_dir).unwrap();

    let engram_data = embeddenator_fs::EmbrFS::load_engram(&engram_path).unwrap();
    let manifest_data = embeddenator_fs::EmbrFS::load_manifest(&manifest_path).unwrap();
    let config = embeddenator_vsa::ReversibleVSAConfig::default();

    embeddenator_fs::EmbrFS::extract(&engram_data, &manifest_data, &output_dir, false, &config)
        .unwrap();

    // Verify extracted files
    let extracted_new = fs::read(output_dir.join("new.txt")).unwrap();
    assert_eq!(extracted_new, b"New content");

    let extracted_file1 = fs::read(output_dir.join("file1.txt")).unwrap();
    assert_eq!(extracted_file1, b"Modified content");
}
