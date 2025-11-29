# CLI Reference

Decy provides a command-line interface for C-to-Rust transpilation with various options for controlling the transpilation process.

## Commands

### `decy transpile`

Transpile a single C source file to Rust.

```bash
decy transpile [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>` - Path to the C source file

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `-o, --output <FILE>` | Output file path (stdout if omitted) | stdout |
| `--oracle` | Enable CITL oracle for error correction | disabled |
| `--oracle-threshold <FLOAT>` | Confidence threshold (0.0-1.0) | 0.7 |
| `--auto-fix` | Automatically apply oracle fixes | disabled |
| `--capture` | Capture verified fix patterns for learning | disabled |
| `--import-patterns <FILE>` | Import patterns from .apr file | none |
| `--oracle-report <FORMAT>` | Output metrics (json, markdown, prometheus) | none |

**Examples:**

```bash
# Basic transpilation
decy transpile hello.c -o hello.rs

# Transpile with oracle assistance
decy transpile --oracle --auto-fix input.c -o output.rs

# Full oracle workflow with pattern capture
decy transpile --oracle --auto-fix --capture \
    --import-patterns base.apr \
    --oracle-report json \
    input.c -o output.rs
```

### `decy transpile-project`

Transpile an entire C project directory.

```bash
decy transpile-project [OPTIONS] <DIR> -o <OUTPUT_DIR>
```

**Arguments:**
- `<DIR>` - Path to the C project directory
- `-o, --output <DIR>` - Output directory for transpiled files

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--no-cache` | Disable incremental caching | enabled |
| `-v, --verbose` | Show per-file progress | disabled |
| `-q, --quiet` | Suppress progress output | disabled |
| `--dry-run` | Show what would be done without writing | disabled |
| `--stats` | Show summary statistics after transpilation | disabled |
| `--oracle` | Enable CITL oracle for error correction | disabled |
| `--oracle-threshold <FLOAT>` | Confidence threshold (0.0-1.0) | 0.7 |
| `--auto-fix` | Automatically apply oracle fixes | disabled |
| `--capture` | Capture verified fix patterns for learning | disabled |
| `--import-patterns <FILE>` | Import patterns from .apr file | none |
| `--oracle-report <FORMAT>` | Output metrics (json, markdown, prometheus) | none |

**Examples:**

```bash
# Basic project transpilation
decy transpile-project ./my-c-project -o ./rust-output

# With caching disabled and verbose output
decy transpile-project --no-cache --verbose ./src -o ./rust-src

# Full oracle workflow for CI
decy transpile-project --oracle --auto-fix --capture \
    --oracle-report markdown \
    ./project -o ./output > ci-report.md
```

### `decy check-project`

Check a project and show the build order without transpiling (dry-run).

```bash
decy check-project <DIR>
```

**Arguments:**
- `<DIR>` - Path to the C project directory

**Example:**

```bash
decy check-project ./my-c-project
```

### `decy cache-stats`

Show cache statistics for a project.

```bash
decy cache-stats <DIR>
```

**Arguments:**
- `<DIR>` - Path to the project directory

**Example:**

```bash
decy cache-stats ./my-c-project
```

### `decy audit`

Audit unsafe code in Rust files.

```bash
decy audit [OPTIONS] <FILE>
```

**Arguments:**
- `<FILE>` - Path to the Rust source file to audit

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `-v, --verbose` | Show detailed information for each unsafe block | disabled |

**Example:**

```bash
# Basic audit
decy audit output.rs

# Detailed audit
decy audit --verbose output.rs
```

### `decy repl`

Start interactive REPL mode for experimenting with transpilation.

```bash
decy repl
```

**REPL Commands:**
- Enter C code to transpile
- Use multi-line mode for function definitions
- Type `exit` or `quit` to leave

### `decy oracle`

Oracle management commands for training and maintenance.

```bash
decy oracle <COMMAND>
```

**Subcommands:**

#### `decy oracle bootstrap`

Bootstrap the oracle with seed patterns for cold start. This command loads predefined patterns for common C→Rust transpilation errors, solving the cold start problem where the oracle has no patterns to learn from.

```bash
decy oracle bootstrap [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--dry-run` | Show available patterns without saving | disabled |

**Example:**
```bash
# Preview bootstrap patterns
decy oracle bootstrap --dry-run

# Bootstrap the oracle
decy oracle bootstrap
```

**Bootstrap Patterns Include:**
- **E0308** (Type mismatch): pointer_to_reference, type_coercion, array_to_slice
- **E0133** (Unsafe required): unsafe_deref, unsafe_extern
- **E0382** (Use after move): clone_before_move, borrow_instead_of_move
- **E0499** (Multiple mutable borrows): sequential_mutable_borrow, use_stdlib_method
- **E0506** (Assign to borrowed): reorder_borrow
- **E0597** (Value not live long enough): extend_lifetime, return_owned
- **E0515** (Return reference to local): return_owned, clone_return

#### `decy oracle seed`

Import patterns from another project's .apr file.

```bash
decy oracle seed --from <FILE>
```

**Options:**
| Flag | Description | Required |
|------|-------------|----------|
| `--from <FILE>` | Path to .apr file to import | yes |

**Example:**
```bash
# Seed from depyler patterns
decy oracle seed --from ../depyler/depyler.apr
```

#### `decy oracle stats`

Show oracle statistics.

```bash
decy oracle stats [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--format <FORMAT>` | Output format (json, markdown, prometheus) | markdown |

**Examples:**
```bash
# Default markdown output
decy oracle stats

# JSON for CI pipelines
decy oracle stats --format json

# Prometheus for monitoring
decy oracle stats --format prometheus
```

#### `decy oracle retire`

Retire obsolete patterns for oracle hygiene.

```bash
decy oracle retire [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--dry-run` | Preview retirements without applying | disabled |
| `--archive-path <FILE>` | Save retired patterns to file | none |

**Examples:**
```bash
# Preview retirements
decy oracle retire --dry-run

# Apply retirements with archive
decy oracle retire --archive-path ./retired-patterns.apr
```

#### `decy oracle validate`

Validate oracle on a corpus with diversity analysis.

```bash
decy oracle validate <DIR>
```

**Arguments:**
- `<DIR>` - Path to corpus directory containing C files

**Example:**
```bash
# Validate on reprorusted-c-cli corpus
decy oracle validate ../reprorusted-c-cli/coreutils/
```

**Output includes:**
- Corpus diversity analysis (file count, LOC, C construct coverage)
- Transpilation success/failure rates
- Error distribution by category
- Oracle query metrics

#### `decy oracle query`

Query oracle for fix patterns for a specific rustc error code.

```bash
decy oracle query --error <CODE> [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--error <CODE>` | Rustc error code (e.g., E0308, E0382) | required |
| `--context <TEXT>` | Optional code context for better matching | none |
| `--format <FORMAT>` | Output format (text, json) | text |

**Examples:**
```bash
# Query patterns for type mismatch
decy oracle query --error E0308

# Query with context
decy oracle query --error E0382 --context "let x = value; use(x);"

# JSON output for tooling
decy oracle query --error E0308 --format json
```

#### `decy oracle train`

Train oracle on a C corpus using CITL feedback loop.

```bash
decy oracle train --corpus <DIR> [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--corpus <DIR>` | Path to corpus directory containing C files | required |
| `--tier <TIER>` | Training tier: P0 (simple), P1 (I/O), P2 (complex) | P0 |
| `--dry-run` | Preview training without saving patterns | disabled |

**Examples:**
```bash
# Train on P0 tier
decy oracle train --corpus ./c-corpus --tier P0

# Preview training
decy oracle train --corpus ./c-corpus --dry-run
```

#### `decy oracle generate-traces`

Generate Golden Traces from C corpus for model training.

```bash
decy oracle generate-traces --corpus <DIR> --output <FILE> [OPTIONS]
```

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--corpus <DIR>` | Path to corpus directory containing C files | required |
| `--output <FILE>` | Output file path for JSONL traces | required |
| `--tier <TIER>` | Training tier: P0 (simple), P1 (I/O), P2 (complex) | P0 |
| `--dry-run` | Preview generation without writing output | disabled |

**Examples:**
```bash
# Generate P0 traces
decy oracle generate-traces \
    --corpus ./c-corpus \
    --output ./traces.jsonl \
    --tier P0

# Preview generation
decy oracle generate-traces \
    --corpus ./c-corpus \
    --output ./traces.jsonl \
    --dry-run
```

#### `decy oracle export`

Export patterns to dataset format for HuggingFace.

```bash
decy oracle export <FILE> [OPTIONS]
```

**Arguments:**
- `<FILE>` - Output file path

**Options:**
| Flag | Description | Default |
|------|-------------|---------|
| `--format <FORMAT>` | Export format: jsonl, chatml, alpaca, parquet | jsonl |
| `--with-card` | Also generate dataset card (README.md) | disabled |

**Examples:**
```bash
# Export to JSONL
decy oracle export ./patterns.jsonl --format jsonl

# Export with dataset card
decy oracle export ./dataset.jsonl --format jsonl --with-card

# Export to ChatML format
decy oracle export ./patterns.chatml --format chatml
```

## Oracle Integration

The oracle integration uses entrenar's CITL (Compiler-in-the-Loop Training) system to automatically fix rustc errors during transpilation.

### How It Works

1. **Query**: When rustc produces an error, the oracle queries accumulated fix patterns
2. **Suggest**: Patterns matching the error code and context are ranked by confidence
3. **Apply**: If `--auto-fix` is enabled and confidence exceeds threshold, the fix is applied
4. **Verify**: The code is recompiled to verify the fix worked
5. **Capture**: If `--capture` is enabled and the fix succeeded, the pattern is saved

### Oracle Flags

| Flag | Purpose |
|------|---------|
| `--oracle` | Enable the CITL oracle system |
| `--oracle-threshold` | Minimum confidence score (0.0-1.0) to apply fixes |
| `--auto-fix` | Automatically apply suggested fixes |
| `--capture` | Save verified fixes to pattern library |
| `--import-patterns` | Load patterns from another project's .apr file |
| `--oracle-report` | Output metrics in specified format |

### Report Formats

**JSON** - Machine-readable for CI/CD pipelines:
```bash
decy transpile --oracle --oracle-report json input.c
```

**Markdown** - Human-readable for PR comments:
```bash
decy transpile --oracle --oracle-report markdown input.c > report.md
```

**Prometheus** - Metrics for monitoring systems:
```bash
decy transpile --oracle --oracle-report prometheus input.c
```

### Cross-Project Pattern Transfer

Patterns can be shared between projects using .apr files:

```bash
# Export patterns from project A
decy transpile-project --oracle --capture ./project-a -o ./out-a

# Import patterns to project B
decy transpile-project --oracle --import-patterns ./project-a.apr \
    ./project-b -o ./out-b
```

Transferable error codes (ownership/lifetime patterns):
- E0382 - Borrow of moved value
- E0499 - Cannot borrow as mutable more than once
- E0506 - Cannot assign to borrowed value
- E0597 - Borrowed value does not live long enough
- E0515 - Cannot return reference to local variable

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Transpilation error |
| 2 | Invalid arguments |
| 3 | File I/O error |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `DECY_CACHE_DIR` | Override default cache directory |
| `RUST_LOG` | Set logging level (debug, info, warn, error) |

## Feature Flags

Build decy with optional features:

```bash
# Build with oracle support
cargo build --features oracle

# Build release with all features
cargo build --release --features oracle
```
