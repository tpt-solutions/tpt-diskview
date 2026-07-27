use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tpt_diskview_cleanup::{detect_duplicates, detect_stale_docker_volumes, temp::detect_temp_files, temp::detect_browser_cache};

#[test]
fn test_detect_temp_files() {
    let dir = tempdir().unwrap();
    
    // Create temp directories that the detector looks for
    let temp_dir = dir.path().join("Temp");
    fs::create_dir_all(&temp_dir).unwrap();
    
    // Create some temp files
    fs::write(temp_dir.join("temp1.tmp"), "temp content").unwrap();
    fs::write(temp_dir.join("temp2.tmp"), "more temp content").unwrap();
    fs::write(temp_dir.join("cache.dat"), "cache data").unwrap();
    
    // Create a regular file (should not be detected)
    fs::write(dir.path().join("regular.txt"), "regular file").unwrap();
    
    let candidates = detect_temp_files(dir.path());
    
    // Should find temp files
    assert!(!candidates.is_empty());
    for candidate in &candidates {
        assert!(candidate.path.contains("Temp") || candidate.path.contains("temp") || candidate.path.contains("cache"));
    }
}

#[test]
fn test_detect_browser_cache() {
    let dir = tempdir().unwrap();
    
    // Create browser cache directories that the detector looks for
    let chrome_cache = dir.path().join("Cache");
    fs::create_dir_all(&chrome_cache).unwrap();
    fs::write(chrome_cache.join("cache_file_1"), "chrome cache").unwrap();
    
    let firefox_cache = dir.path().join("cache");
    fs::create_dir_all(&firefox_cache).unwrap();
    fs::write(firefox_cache.join("cache_file_2"), "firefox cache").unwrap();
    
    let candidates = detect_browser_cache(dir.path());
    
    // Should find browser cache
    assert!(!candidates.is_empty());
    for candidate in &candidates {
        assert!(candidate.path.contains("Cache") || candidate.path.contains("cache"));
    }
}

#[test]
fn test_detect_duplicates() {
    let dir = tempdir().unwrap();
    
    // Create duplicate files
    let file1 = dir.path().join("file1.txt");
    let file2 = dir.path().join("file2.txt");
    let file3 = dir.path().join("file3.txt");
    
    fs::write(&file1, "duplicate content").unwrap();
    fs::write(&file2, "duplicate content").unwrap();
    fs::write(&file3, "unique content").unwrap();
    
    let candidates = detect_duplicates(dir.path());
    
    // Should find 1 duplicate (file2 is duplicate of file1)
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].size, "duplicate content".len() as u64);
}

#[test]
fn test_detect_duplicates_empty_dir() {
    let dir = tempdir().unwrap();
    let candidates = detect_duplicates(dir.path());
    assert!(candidates.is_empty());
}

#[test]
fn test_detect_duplicates_no_duplicates() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(dir.path().join("file2.txt"), "content2").unwrap();
    fs::write(dir.path().join("file3.txt"), "content3").unwrap();
    
    let candidates = detect_duplicates(dir.path());
    assert!(candidates.is_empty());
}

#[test]
fn test_detect_stale_docker_volumes_windows() {
    let dir = tempdir().unwrap();
    
    // Create Docker directory structure for Windows
    let docker_path = dir.path().join("AppData").join("Local").join("Docker");
    let volumes_path = docker_path.join("volumes");
    fs::create_dir_all(&volumes_path).unwrap();
    fs::write(volumes_path.join("volume1"), "docker volume data").unwrap();
    
    let candidates = detect_stale_docker_volumes(dir.path());
    
    // Should find Docker volumes
    assert!(!candidates.is_empty());
    for candidate in &candidates {
        assert!(candidate.path.contains("volumes") || candidate.path.contains("overlay2") || candidate.path.contains("image"));
    }
}

#[test]
fn test_detect_stale_docker_volumes_linux() {
    // This test only runs on Linux
    if !cfg!(target_os = "linux") {
        return;
    }
    
    let dir = tempdir().unwrap();
    
    // Create Docker directory structure for Linux
    let docker_path = dir.path().join("var").join("lib").join("docker");
    let volumes_path = docker_path.join("volumes");
    fs::create_dir_all(&volumes_path).unwrap();
    fs::write(volumes_path.join("volume1"), "docker volume data").unwrap();
    
    let candidates = detect_stale_docker_volumes(dir.path());
    
    // Should find Docker volumes
    assert!(!candidates.is_empty());
    for candidate in &candidates {
        assert!(candidate.path.contains("volumes") || candidate.path.contains("overlay2") || candidate.path.contains("image"));
    }
}

#[test]
fn test_detect_stale_docker_volumes_no_docker() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("regular.txt"), "regular file").unwrap();
    
    let candidates = detect_stale_docker_volumes(dir.path());
    assert!(candidates.is_empty());
}

#[test]
fn test_detect_temp_files_empty_dir() {
    let dir = tempdir().unwrap();
    let candidates = detect_temp_files(dir.path());
    assert!(candidates.is_empty());
}

#[test]
fn test_detect_browser_cache_empty_dir() {
    let dir = tempdir().unwrap();
    let candidates = detect_browser_cache(dir.path());
    assert!(candidates.is_empty());
}