//! CLI tool for C-to-Rust transpilation with EXTREME quality standards.

#![warn(clippy::all)]
#![deny(unsafe_code)]

mod oracle_integration;
mod repl;

use oracle_integration::{OracleOptions, OracleTranspileResult};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

/// Decy: C-to-Rust Transpiler with EXTREME Quality Standards
#[derive(Parser, Debug)]
#[command(name = "decy")]
#[command(version = "0.2.0")]
#[command(about = "Transpile C code to safe Rust with minimal unsafe blocks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Transpile a C source file to Rust
    Transpile {
        /// Path to the C source file
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Output file (default: stdout)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Enable CITL oracle for error correction
        #[arg(long)]
        oracle: bool,

        /// Confidence threshold for oracle suggestions (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        oracle_threshold: f32,

        /// Automatically apply oracle fixes
        #[arg(long)]
        auto_fix: bool,

        /// Capture verified fix patterns for learning
        #[arg(long)]
        capture: bool,

        /// Import patterns from another .apr file (cross-project transfer)
        #[arg(long, value_name = "FILE")]
        import_patterns: Option<PathBuf>,

        /// Output oracle metrics report (json, markdown, prometheus)
        #[arg(long, value_name = "FORMAT")]
        oracle_report: Option<String>,
    },
    /// Transpile an entire C project (directory)
    TranspileProject {
        /// Path to the project directory
        #[arg(value_name = "DIR")]
        input: PathBuf,

        /// Output directory for transpiled files
        #[arg(short, long, value_name = "DIR")]
        output: PathBuf,

        /// Disable caching (default: enabled)
        #[arg(long)]
        no_cache: bool,

        /// Show verbose output (per-file progress)
        #[arg(short, long, conflicts_with = "quiet")]
        verbose: bool,

        /// Suppress progress output
        #[arg(short, long, conflicts_with = "verbose")]
        quiet: bool,

        /// Show what would be done without writing files
        #[arg(long)]
        dry_run: bool,

        /// Show summary statistics after transpilation
        #[arg(long)]
        stats: bool,

        /// Enable CITL oracle for error correction
        #[arg(long)]
        oracle: bool,

        /// Confidence threshold for oracle suggestions (0.0-1.0)
        #[arg(long, default_value = "0.7")]
        oracle_threshold: f32,

        /// Automatically apply oracle fixes
        #[arg(long)]
        auto_fix: bool,

        /// Capture verified fix patterns for learning
        #[arg(long)]
        capture: bool,

        /// Import patterns from another .apr file (cross-project transfer)
        #[arg(long, value_name = "FILE")]
        import_patterns: Option<PathBuf>,

        /// Output oracle metrics report (json, markdown, prometheus)
        #[arg(long, value_name = "FORMAT")]
        oracle_report: Option<String>,
    },
    /// Check project and show build order (dry-run)
    CheckProject {
        /// Path to the project directory
        #[arg(value_name = "DIR")]
        input: PathBuf,
    },
    /// Show cache statistics for a project
    CacheStats {
        /// Path to the project directory
        #[arg(value_name = "DIR")]
        input: PathBuf,
    },
    /// Start interactive REPL mode
    Repl,
    /// Audit unsafe code in Rust files
    Audit {
        /// Path to the Rust source file to audit
        #[arg(value_name = "FILE")]
        input: PathBuf,

        /// Show detailed information for each unsafe block
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Transpile {
            input,
            output,
            oracle,
            oracle_threshold,
            auto_fix,
            capture,
            import_patterns,
            oracle_report,
        }) => {
            let oracle_opts = OracleOptions::new(oracle, Some(oracle_threshold), auto_fix)
                .with_capture(capture)
                .with_import(import_patterns)
                .with_report_format(oracle_report);
            transpile_file(input, output, &oracle_opts)?;
        }
        Some(Commands::TranspileProject {
            input,
            output,
            no_cache,
            verbose,
            quiet,
            dry_run,
            stats,
            oracle,
            oracle_threshold,
            auto_fix,
            capture,
            import_patterns,
            oracle_report,
        }) => {
            let oracle_opts = OracleOptions::new(oracle, Some(oracle_threshold), auto_fix)
                .with_capture(capture)
                .with_import(import_patterns)
                .with_report_format(oracle_report);
            transpile_project(
                input,
                output,
                !no_cache,
                verbose,
                quiet,
                dry_run,
                stats,
                &oracle_opts,
            )?;
        }
        Some(Commands::CheckProject { input }) => {
            check_project(input)?;
        }
        Some(Commands::CacheStats { input }) => {
            show_cache_stats(input)?;
        }
        Some(Commands::Repl) => {
            repl::run()?;
        }
        Some(Commands::Audit { input, verbose }) => {
            audit_file(input, verbose)?;
        }
        None => {
            // No subcommand - show info
            println!("Decy: C-to-Rust Transpiler with EXTREME Quality Standards");
            println!("Version 0.1.0");
            println!();
            println!("Use 'decy --help' for usage information");
            println!("Use 'decy transpile <file>' to transpile C code to Rust");
            println!("Use 'decy transpile-project <dir> -o <output>' for entire projects");
            println!("Use 'decy check-project <dir>' to verify build order");
            println!("Use 'decy cache-stats <dir>' to view cache statistics");
            println!("Use 'decy repl' to start interactive mode");
            println!("Use 'decy audit <file>' to audit unsafe code in Rust files");
        }
    }

    Ok(())
}

fn transpile_file(input: PathBuf, output: Option<PathBuf>, oracle_opts: &OracleOptions) -> Result<()> {
    // Read input file
    let c_code = fs::read_to_string(&input).with_context(|| {
        format!(
            "Failed to read input file: {}\n\nTry: Check that the file exists and is readable\n  or: Verify the file path is correct",
            input.display()
        )
    })?;

    // Get base directory for #include resolution (DECY-056)
    let base_dir = input.parent();

    // Transpile - use oracle if enabled
    let (rust_code, oracle_result) = if oracle_opts.should_use_oracle() {
        let result = oracle_integration::transpile_with_oracle(&c_code, oracle_opts)
            .with_context(|| format!("Oracle-assisted transpilation failed for {}", input.display()))?;
        let code = result.rust_code.clone();
        (code, Some(result))
    } else {
        // Standard transpilation using decy-core with #include support
        let code = decy_core::transpile_with_includes(&c_code, base_dir).with_context(|| {
            format!(
                "Failed to transpile {}\n\nTry: Check if the C code has syntax errors\n  or: Preprocess the file first: gcc -E {} -o preprocessed.c",
                input.display(),
                input.display()
            )
        })?;
        (code, None)
    };

    // DECY-AUDIT-002: Detect if the source has no main function and provide guidance
    let has_main = rust_code.contains("fn main(");

    // Write output
    match output {
        Some(output_path) => {
            fs::write(&output_path, &rust_code).with_context(|| {
                format!("Failed to write output file: {}", output_path.display())
            })?;
            eprintln!(
                "✓ Transpiled {} → {}",
                input.display(),
                output_path.display()
            );

            // Show oracle statistics if used
            if let Some(ref result) = oracle_result {
                print_oracle_stats(result, oracle_opts);
            }

            // DECY-AUDIT-002: Provide compilation guidance for library-only files
            if !has_main {
                eprintln!();
                eprintln!("ℹ Note: No main function found in source.");
                eprintln!("  To compile the output as a library, use:");
                eprintln!("  rustc --crate-type=lib {}", output_path.display());
            }
        }
        None => {
            // Write to stdout
            print!("{}", rust_code);

            // Show oracle statistics if used
            if let Some(ref result) = oracle_result {
                print_oracle_stats(result, oracle_opts);
            }

            // DECY-AUDIT-002: Provide compilation guidance for library-only files
            // Only show this to stderr if writing to stdout
            if !has_main {
                eprintln!();
                eprintln!("ℹ Note: No main function found in source.");
                eprintln!("  To compile the output as a library, use:");
                eprintln!("  rustc --crate-type=lib <output_file>");
            }
        }
    }

    Ok(())
}

fn print_oracle_stats(result: &OracleTranspileResult, opts: &OracleOptions) {
    // Check if we should output in a specific format
    if let Some(ref format) = opts.report_format {
        print_oracle_report(result, format);
        return;
    }

    // Default human-readable output
    eprintln!();
    eprintln!("=== Oracle Statistics ===");
    if result.patterns_imported > 0 {
        eprintln!("Patterns imported: {}", result.patterns_imported);
    }
    eprintln!("Queries: {}", result.oracle_queries);
    eprintln!("Fixes applied: {}", result.fixes_applied);
    eprintln!("Retries: {}", result.retries_used);
    if result.patterns_captured > 0 {
        eprintln!("Patterns captured: {}", result.patterns_captured);
    }
    if result.compilation_success {
        eprintln!("✓ Compilation: SUCCESS");
    } else {
        eprintln!("✗ Compilation: FAILED");
        if !result.remaining_errors.is_empty() {
            eprintln!("Remaining errors: {}", result.remaining_errors.len());
            for err in &result.remaining_errors {
                eprintln!("  - {}", err);
            }
        }
    }
}

fn print_oracle_report(result: &OracleTranspileResult, format: &str) {
    #[cfg(feature = "oracle")]
    {
        use decy_oracle::{CIReport, CIThresholds, OracleMetrics};

        // Build metrics from result
        let mut metrics = OracleMetrics::default();
        metrics.queries = result.oracle_queries as u64;
        metrics.hits = result.fixes_applied as u64; // Approximation
        metrics.misses = (result.oracle_queries - result.fixes_applied) as u64;
        metrics.fixes_applied = result.fixes_applied as u64;
        metrics.fixes_verified = if result.compilation_success {
            result.fixes_applied as u64
        } else {
            0
        };
        metrics.patterns_captured = result.patterns_captured as u64;

        let report = CIReport::from_metrics(metrics, CIThresholds::default());

        match format.to_lowercase().as_str() {
            "json" => println!("{}", report.to_json()),
            "markdown" | "md" => println!("{}", report.to_markdown()),
            "prometheus" | "prom" => {
                let m = &report.metrics;
                println!("{}", m.to_prometheus());
            }
            _ => {
                eprintln!("Unknown report format: {}. Use: json, markdown, prometheus", format);
            }
        }
    }

    #[cfg(not(feature = "oracle"))]
    {
        eprintln!("Oracle report format '{}' requires --features oracle", format);
    }
}

fn audit_file(input: PathBuf, verbose: bool) -> Result<()> {
    // Read Rust file
    let rust_code = fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    // Run unsafe code auditor
    let report = decy_verify::audit_rust_code(&rust_code)
        .with_context(|| format!("Failed to audit {}", input.display()))?;

    // Print header
    println!();
    println!("Unsafe Code Audit Report");
    println!("========================");
    println!("File: {}", input.display());
    println!("Total Lines: {}", report.total_lines);
    println!("Unsafe Lines: {}", report.unsafe_lines);
    println!(
        "Unsafe Density: {:.2}% {}",
        report.unsafe_density_percent,
        if report.meets_density_target() {
            "✅ (Target: <5%)"
        } else {
            "❌ (Target: <5%)"
        }
    );
    println!();

    if report.unsafe_blocks.is_empty() {
        println!("✅ No unsafe blocks found - code is 100% safe!");
        return Ok(());
    }

    println!("Unsafe Blocks Found: {}", report.unsafe_blocks.len());
    println!("Average Confidence: {:.1}/100", report.average_confidence);
    println!();

    // Show high-confidence blocks
    let high_conf = report.high_confidence_blocks();
    if !high_conf.is_empty() {
        println!(
            "⚠️  {} blocks with HIGH confidence for elimination (≥70):",
            high_conf.len()
        );
        println!();
    }

    // List all unsafe blocks
    if verbose {
        println!("Detailed Block Analysis:");
        println!("------------------------");
        for (idx, block) in report.unsafe_blocks.iter().enumerate() {
            println!();
            println!(
                "{}. Line {} [Confidence: {}/100 - {}]",
                idx + 1,
                if block.line > 0 {
                    block.line.to_string()
                } else {
                    "N/A".to_string()
                },
                block.confidence,
                if block.confidence >= 70 {
                    "HIGH"
                } else if block.confidence >= 40 {
                    "MEDIUM"
                } else {
                    "LOW"
                }
            );
            println!("   Pattern: {:?}", block.pattern);
            println!("   Suggestion: {}", block.suggestion);
        }
    } else {
        println!("Summary by Confidence:");
        let high = report
            .unsafe_blocks
            .iter()
            .filter(|b| b.confidence >= 70)
            .count();
        let medium = report
            .unsafe_blocks
            .iter()
            .filter(|b| b.confidence >= 40 && b.confidence < 70)
            .count();
        let low = report
            .unsafe_blocks
            .iter()
            .filter(|b| b.confidence < 40)
            .count();

        println!("  HIGH (≥70):   {} blocks - likely can be eliminated", high);
        println!(
            "  MEDIUM (40-69): {} blocks - review possible alternatives",
            medium
        );
        println!("  LOW (<40):    {} blocks - may require unsafe", low);
        println!();
        println!("Use --verbose flag to see detailed information for each block");
    }

    println!();
    println!("---");
    println!("Recommendation: Focus on eliminating HIGH confidence blocks first.");
    println!();

    Ok(())
}

fn transpile_project(
    input_dir: PathBuf,
    output_dir: PathBuf,
    use_cache: bool,
    verbose: bool,
    quiet: bool,
    dry_run: bool,
    stats: bool,
    oracle_opts: &OracleOptions,
) -> Result<()> {
    use decy_core::{DependencyGraph, TranspilationCache};
    use indicatif::{ProgressBar, ProgressStyle};
    use std::time::Instant;
    use walkdir::WalkDir;

    // Validate input directory exists
    if !input_dir.exists() {
        anyhow::bail!(
            "Input directory not found: {}\n\nTry: Check the path is correct\n  or: Use 'decy check-project <dir>' to verify project structure",
            input_dir.display()
        );
    }

    // Create output directory if needed (unless dry-run)
    if !dry_run {
        fs::create_dir_all(&output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;
    }

    // Find all C files
    let c_files: Vec<PathBuf> = WalkDir::new(&input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("c"))
        .map(|e| e.path().to_path_buf())
        .collect();

    if c_files.is_empty() {
        if !quiet {
            println!("No C files found in {}", input_dir.display());
        }
        return Ok(());
    }

    if dry_run && !quiet {
        println!("DRY RUN MODE - No files will be written");
        println!();
    }

    if !quiet {
        println!("Found {} C files to transpile", c_files.len());
        println!();
    }

    // Initialize cache (unless dry-run)
    let cache_dir = input_dir.join(".decy").join("cache");
    if !dry_run {
        fs::create_dir_all(&cache_dir)?;
    }

    let mut cache = if use_cache && !dry_run {
        TranspilationCache::load(&cache_dir)?
    } else {
        TranspilationCache::new()
    };

    // Build dependency graph (simplified - actual implementation in decy-core)
    let mut dep_graph = DependencyGraph::new();
    for file in &c_files {
        dep_graph.add_file(file);
    }

    // Get build order
    let build_order = dep_graph
        .topological_sort()
        .with_context(|| "Failed to compute build order (circular dependencies?)")?;

    // Setup progress bar (unless quiet mode)
    let pb = if quiet {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(c_files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb
    };

    let start_time = Instant::now();
    let mut transpiled_count = 0;
    let mut cached_count = 0;
    let mut total_lines = 0;

    // Transpile files in dependency order
    for file_path in build_order {
        let relative_path = file_path.strip_prefix(&input_dir).unwrap_or(&file_path);
        pb.set_message(format!("Transpiling {}", relative_path.display()));

        // Check cache
        if use_cache {
            if let Some(_cached) = cache.get(&file_path) {
                if verbose {
                    println!("✓ Cached: {}", relative_path.display());
                }
                pb.set_message(format!("✓ Cached {}", relative_path.display()));
                cached_count += 1;
                pb.inc(1);
                continue;
            }
        }

        // Read C code
        let c_code = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        if dry_run {
            // Dry run mode - always show what would be done (that's the point of dry-run!)
            if !quiet {
                println!("Would transpile: {}", relative_path.display());
            }
            pb.set_message(format!("Would transpile {}", relative_path.display()));
            pb.inc(1);
            continue;
        }

        // Transpile
        let rust_code = decy_core::transpile(&c_code)
            .with_context(|| format!("Failed to transpile {}", file_path.display()))?;

        total_lines += rust_code.lines().count();

        // Compute output path (preserve directory structure)
        let output_path = output_dir.join(relative_path).with_extension("rs");

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write output
        fs::write(&output_path, &rust_code)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;

        if verbose {
            println!(
                "✓ Transpiled: {} → {}",
                relative_path.display(),
                output_path.display()
            );
        }

        // Update cache
        if use_cache {
            let transpiled = decy_core::TranspiledFile {
                source_path: file_path.clone(),
                rust_code: rust_code.clone(),
                dependencies: vec![], // Would be populated by actual parser
                functions_exported: vec![], // Would be populated by actual parser
                ffi_declarations: String::new(), // Would be populated by actual parser
            };
            cache.insert(&file_path, &transpiled);
        }

        transpiled_count += 1;
        pb.inc(1);
    }

    pb.finish_with_message("Done");

    // Save cache (unless dry-run)
    if use_cache && !dry_run {
        cache.save()?;
    }

    let elapsed = start_time.elapsed();

    if !quiet {
        println!();
        if dry_run {
            println!(
                "✓ Dry run complete - {} files checked in {:.2}s",
                c_files.len(),
                elapsed.as_secs_f64()
            );
        } else {
            println!(
                "✓ Transpiled {} files in {:.2}s",
                transpiled_count,
                elapsed.as_secs_f64()
            );
        }
    }

    // Show statistics if requested or if verbose
    if (stats || verbose) && !quiet {
        println!();
        println!("=== Statistics ===");
        println!("Files found: {}", c_files.len());
        println!("Files transpiled: {}", transpiled_count);
        if cached_count > 0 {
            println!("Files cached: {}", cached_count);
        }
        if total_lines > 0 {
            println!("Lines generated: {}", total_lines);
        }
        println!("Time elapsed: {:.2}s", elapsed.as_secs_f64());

        if use_cache {
            let cache_stats = cache.statistics();
            println!();
            println!("=== Cache Statistics ===");
            println!("Cache hits: {}", cache_stats.hits);
            println!("Cache misses: {}", cache_stats.misses);
            if cache_stats.hits + cache_stats.misses > 0 {
                let hit_rate = (cache_stats.hits as f64
                    / (cache_stats.hits + cache_stats.misses) as f64)
                    * 100.0;
                println!("Hit rate: {:.1}%", hit_rate);
                let speedup = if cache_stats.misses > 0 {
                    (cache_stats.hits + cache_stats.misses) as f64 / cache_stats.misses as f64
                } else {
                    1.0
                };
                println!("Estimated speedup: {:.1}x", speedup);
            }
        }
    }

    if !quiet && !dry_run {
        println!();
        println!("Output directory: {}", output_dir.display());
    }

    Ok(())
}

fn check_project(input_dir: PathBuf) -> Result<()> {
    use decy_core::DependencyGraph;
    use walkdir::WalkDir;

    // Validate input directory
    if !input_dir.exists() {
        anyhow::bail!("Input directory not found: {}", input_dir.display());
    }

    println!("Checking project: {}", input_dir.display());
    println!();

    // Find all C files
    let c_files: Vec<PathBuf> = WalkDir::new(&input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("c"))
        .map(|e| e.path().to_path_buf())
        .collect();

    if c_files.is_empty() {
        println!("No C files found.");
        return Ok(());
    }

    println!("Found {} C files:", c_files.len());
    for file in &c_files {
        let relative = file.strip_prefix(&input_dir).unwrap_or(file);
        println!("  - {}", relative.display());
    }
    println!();

    // Build dependency graph
    let mut dep_graph = DependencyGraph::new();
    for file in &c_files {
        dep_graph.add_file(file);
    }

    // Check for circular dependencies
    match dep_graph.topological_sort() {
        Ok(build_order) => {
            println!("✓ No circular dependencies detected");
            println!();
            println!("Build order:");
            for (idx, file) in build_order.iter().enumerate() {
                let relative = file.strip_prefix(&input_dir).unwrap_or(file);
                println!("  {}. {}", idx + 1, relative.display());
            }
        }
        Err(e) => {
            println!("❌ Circular dependencies detected:");
            println!("   {}", e);
            return Err(e);
        }
    }

    println!();
    println!("Project is ready for transpilation.");

    Ok(())
}

fn show_cache_stats(input_dir: PathBuf) -> Result<()> {
    use decy_core::TranspilationCache;

    // Validate input directory
    if !input_dir.exists() {
        anyhow::bail!("Input directory not found: {}", input_dir.display());
    }

    let cache_dir = input_dir.join(".decy").join("cache");

    if !cache_dir.exists() {
        println!("No cache found at {}", input_dir.display());
        println!();
        println!("Run 'decy transpile-project' with caching enabled to create a cache.");
        return Ok(());
    }

    // Load cache
    let cache = TranspilationCache::load(&cache_dir)?;

    println!("Cache Statistics");
    println!("================");
    println!("Location: {}", input_dir.join(".decy/cache").display());
    println!();

    let stats = cache.statistics();
    let total = stats.hits + stats.misses;

    println!("Total files cached: {}", stats.total_files);
    println!("Total requests: {}", total);
    println!("Cache hits: {}", stats.hits);
    println!("Cache misses: {}", stats.misses);

    if total > 0 {
        let hit_rate = (stats.hits as f64 / total as f64) * 100.0;
        println!("Hit rate: {:.1}%", hit_rate);

        if hit_rate > 80.0 {
            println!();
            println!("✓ Excellent cache performance!");
        } else if hit_rate > 50.0 {
            println!();
            println!("ℹ Good cache performance.");
        } else if hit_rate > 0.0 {
            println!();
            println!("⚠ Low cache hit rate - files may be changing frequently.");
        }
    } else {
        println!();
        println!("No cache usage recorded yet.");
    }

    Ok(())
}
