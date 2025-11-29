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
