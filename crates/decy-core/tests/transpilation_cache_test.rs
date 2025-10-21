//! Transpilation cache tests (DECY-049 RED phase)
//!
//! These tests define the API for caching transpilation results to avoid
//! re-transpiling unchanged files. Uses SHA-256 hashing for change detection.
//!
//! Goal: 10-20x speedup on cache hits (2ms → 0.1ms per file).

use decy_core::{ProjectContext, TranspilationCache};
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper: Create temporary C file with content
fn create_temp_c_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, content).expect("Failed to write temp file");
    path
}

#[test]
fn test_cache_stores_transpiled_file() {
    // Test: Cache stores TranspiledFile for future retrieval
    let temp = TempDir::new().unwrap();
    let c_file = create_temp_c_file(&temp, "test.c", "int add(int a, int b) { return a + b; }");

    let mut cache = TranspilationCache::new();
    let context = ProjectContext::new();

    // Transpile and cache
    let transpiled = decy_core::transpile_file(&c_file, &context).expect("Should transpile");
    cache.insert(&c_file, &transpiled);

    // Retrieve from cache
    let cached = cache.get(&c_file);
    assert!(cached.is_some(), "Should find cached file");

    let cached_file = cached.unwrap();
    assert_eq!(cached_file.source_path, transpiled.source_path);
    assert_eq!(cached_file.rust_code, transpiled.rust_code);
}

#[test]
fn test_cache_hit_on_unchanged_file() {
    // Test: Cache returns cached result when file hasn't changed
    let temp = TempDir::new().unwrap();
    let c_file = create_temp_c_file(
        &temp,
        "test.c",
        "int multiply(int x, int y) { return x * y; }",
    );

    let mut cache = TranspilationCache::new();
    let context = ProjectContext::new();

    // First transpilation - cache miss
    let transpiled1 = decy_core::transpile_file(&c_file, &context).expect("Should transpile");
    cache.insert(&c_file, &transpiled1);

    // Second access - cache hit (file unchanged)
    let cached = cache.get(&c_file);
    assert!(cached.is_some(), "Should hit cache");

    let cached_file = cached.unwrap();
    assert_eq!(cached_file.rust_code, transpiled1.rust_code);
}

#[test]
fn test_cache_miss_on_changed_file() {
    // Test: Cache detects file changes and invalidates entry
    let temp = TempDir::new().unwrap();
    let c_file = create_temp_c_file(&temp, "test.c", "int foo() { return 1; }");

    let mut cache = TranspilationCache::new();
    let context = ProjectContext::new();

    // First transpilation
    let transpiled1 = decy_core::transpile_file(&c_file, &context).expect("Should transpile");
    cache.insert(&c_file, &transpiled1);

    // Modify file
    std::fs::write(&c_file, "int foo() { return 2; }").expect("Should write");

    // Cache should detect change
    let cached = cache.get(&c_file);
    assert!(
        cached.is_none(),
        "Should detect file change and invalidate cache"
    );
}

#[test]
fn test_cache_invalidation_on_dependency_change() {
    // Test: Cache invalidates when a dependency changes
    // Note: We manually set up dependencies for this test since the parser
    // doesn't process #include directives
    let temp = TempDir::new().unwrap();

    // Create header file
    let header = create_temp_c_file(&temp, "lib.h", "int helper();");

    // Create implementation file (valid C without #include)
    let impl_file = create_temp_c_file(&temp, "main.c", "int main() { return 0; }");

    let mut cache = TranspilationCache::new();
    let context = ProjectContext::new();

    // Transpile and cache with manually added dependency
    let mut transpiled = decy_core::transpile_file(&impl_file, &context).expect("Should transpile");
    transpiled.dependencies = vec![header.clone()]; // Manually add dependency
    cache.insert(&impl_file, &transpiled);

    // Verify cache hit before change
    let cached = cache.get(&impl_file);
    assert!(
        cached.is_some(),
        "Should have cached entry before dependency change"
    );

    // Modify dependency (header)
    std::fs::write(&header, "int helper(); // changed").expect("Should write");

    // Cache should invalidate main.c because dependency changed
    let cached = cache.get(&impl_file);
    assert!(
        cached.is_none(),
        "Should invalidate cache when dependency changes"
    );
}

#[test]
fn test_cache_persistence_to_disk() {
    // Test: Cache persists to disk and loads across runs
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path().join(".decy/cache");
    let c_file = create_temp_c_file(&temp, "test.c", "int value() { return 42; }");

    let context = ProjectContext::new();

    // First run: Create cache and persist
    {
        let mut cache = TranspilationCache::with_directory(&cache_dir);
        let transpiled = decy_core::transpile_file(&c_file, &context).expect("Should transpile");
        cache.insert(&c_file, &transpiled);
        cache.save().expect("Should save cache to disk");
    }

    // Second run: Load cache from disk
    {
        let mut cache = TranspilationCache::load(&cache_dir).expect("Should load cache from disk");
        let cached = cache.get(&c_file);
        assert!(cached.is_some(), "Should load cached file from disk");
    }
}

#[test]
fn test_cache_statistics() {
    // Test: Cache tracks hits and misses
    let temp = TempDir::new().unwrap();
    let file1 = create_temp_c_file(&temp, "file1.c", "int a() { return 1; }");
    let file2 = create_temp_c_file(&temp, "file2.c", "int b() { return 2; }");

    let mut cache = TranspilationCache::new();
    let context = ProjectContext::new();

    // First access - miss
    let transpiled1 = decy_core::transpile_file(&file1, &context).expect("Should transpile");
    cache.insert(&file1, &transpiled1);
    let _ = cache.get(&file1); // hit

    // Second file - miss
    let transpiled2 = decy_core::transpile_file(&file2, &context).expect("Should transpile");
    cache.insert(&file2, &transpiled2);
    let _ = cache.get(&file2); // hit

    // Third access to file1 - hit
    let _ = cache.get(&file1); // hit

    let stats = cache.statistics();
    assert_eq!(stats.hits, 3, "Should count 3 cache hits");
    assert_eq!(
        stats.misses, 0,
        "Should count 0 misses (insert doesn't count)"
    );
    assert_eq!(stats.total_files, 2, "Should track 2 files");
}

#[test]
fn test_cache_sha256_hash_computation() {
    // Test: Cache uses SHA-256 to detect file changes
    let temp = TempDir::new().unwrap();
    let c_file = create_temp_c_file(&temp, "test.c", "int hash_test() { return 1; }");

    let cache = TranspilationCache::new();

    // Compute hash
    let hash1 = cache.compute_hash(&c_file).expect("Should compute hash");
    assert_eq!(hash1.len(), 64, "SHA-256 hash should be 64 hex characters");

    // Same file, same hash
    let hash2 = cache.compute_hash(&c_file).expect("Should compute hash");
    assert_eq!(hash1, hash2, "Hash should be deterministic");

    // Modify file
    std::fs::write(&c_file, "int hash_test() { return 2; }").expect("Should write");

    // Different hash
    let hash3 = cache.compute_hash(&c_file).expect("Should compute hash");
    assert_ne!(hash1, hash3, "Hash should change when file content changes");
}

#[test]
fn test_cache_clears_all_entries() {
    // Test: Cache can be cleared
    let temp = TempDir::new().unwrap();
    let c_file = create_temp_c_file(&temp, "test.c", "int clear_test() { return 0; }");

    let mut cache = TranspilationCache::new();
    let context = ProjectContext::new();

    // Add entry
    let transpiled = decy_core::transpile_file(&c_file, &context).expect("Should transpile");
    cache.insert(&c_file, &transpiled);
    assert!(cache.get(&c_file).is_some(), "Should have cached entry");

    // Clear cache
    cache.clear();

    // Verify cleared
    assert!(
        cache.get(&c_file).is_none(),
        "Cache should be empty after clear"
    );
    let stats = cache.statistics();
    assert_eq!(stats.total_files, 0, "Should have 0 files after clear");
}

#[test]
fn test_cache_with_multiple_files() {
    // Test: Cache handles multiple files efficiently
    let temp = TempDir::new().unwrap();
    let context = ProjectContext::new();

    let mut cache = TranspilationCache::new();
    let mut files = Vec::new();

    // Create and cache 10 files
    for i in 0..10 {
        let content = format!("int file{}() {{ return {}; }}", i, i);
        let file = create_temp_c_file(&temp, &format!("file{}.c", i), &content);

        let transpiled = decy_core::transpile_file(&file, &context).expect("Should transpile");
        cache.insert(&file, &transpiled);
        files.push(file);
    }

    // Verify all cached
    for file in &files {
        assert!(cache.get(file).is_some(), "Should find cached file");
    }

    let stats = cache.statistics();
    assert_eq!(stats.total_files, 10, "Should track 10 files");
}
