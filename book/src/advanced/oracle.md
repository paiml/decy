# Oracle Integration (CITL)

Decy integrates with entrenar's CITL (Compiler-in-the-Loop Training) system to automatically learn and apply fixes for common rustc errors during C-to-Rust transpilation.

## Overview

The oracle system implements the **Jidoka** principle from the Toyota Way: automation with human intelligence. When transpilation produces rustc errors, the oracle:

1. Queries accumulated fix patterns from previous successful repairs
2. Suggests fixes ranked by confidence score
3. Applies fixes automatically when confidence exceeds threshold
4. Captures verified fixes for future use

This creates a feedback loop where the transpiler becomes more effective over time.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      DECY ORACLE PIPELINE                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  C Source ──► Parser ──► HIR ──► Ownership ──► Codegen ──► Rust │
│                                      │                     │     │
│                                      ▼                     ▼     │
│                              ┌─────────────┐        ┌──────────┐ │
│                              │   ORACLE    │◄───────│  rustc   │ │
│                              │   QUERY     │        │  errors  │ │
│                              └─────────────┘        └──────────┘ │
│                                    │                             │
│                                    ▼                             │
│                           ┌───────────────┐                      │
│                           │ .apr Patterns │                      │
│                           │ (Fix Library) │                      │
│                           └───────────────┘                      │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Decision Categories

The oracle classifies C-to-Rust decisions into categories that map to specific rustc error codes:

### Ownership Inference

| Category | C Pattern | Rust Target | Common Errors |
|----------|-----------|-------------|---------------|
| PointerOwnership | `*T` | `Box<T>`, `&T`, `&mut T` | E0382, E0499, E0506 |
| ArrayOwnership | `T[]` | `Vec<T>`, `&[T]`, `Box<[T]>` | E0382, E0499, E0506 |
| StringOwnership | `char*` | `String`, `&str`, `CString` | E0382, E0308 |

### Lifetime Inference

| Category | Decision | Common Errors |
|----------|----------|---------------|
| LifetimeElision | Elide vs explicit `'a` | E0597, E0515 |
| StructLifetime | Field lifetime annotations | E0597, E0515 |
| ReturnLifetime | Return reference binding | E0515, E0597 |

### Unsafe Minimization

| Category | Decision | Common Errors |
|----------|----------|---------------|
| UnsafeBlock | When `unsafe` is required | E0133 |
| RawPointerCast | `*const T` → `&T` safety | E0133, E0606 |
| NullCheck | `NULL` → `Option<T>` | E0308 |

## Usage

### Basic Oracle Transpilation

```bash
# Enable oracle with default threshold (0.7)
decy transpile --oracle input.c -o output.rs
```

### Auto-Fix Mode

```bash
# Automatically apply fixes above confidence threshold
decy transpile --oracle --auto-fix input.c -o output.rs
```

### Adjust Confidence Threshold

```bash
# Higher threshold = more conservative
decy transpile --oracle --auto-fix --oracle-threshold 0.9 input.c

# Lower threshold = more aggressive
decy transpile --oracle --auto-fix --oracle-threshold 0.5 input.c
```

### Pattern Capture (Learning Mode)

```bash
# Capture verified fixes for future use
decy transpile --oracle --auto-fix --capture input.c -o output.rs
```

When `--capture` is enabled:
1. Fixes that lead to successful compilation are recorded
2. Patterns are saved to the `.apr` pattern library
3. Future transpilations benefit from learned patterns

## Cross-Project Pattern Transfer

The oracle supports sharing patterns between projects (Yokoten principle):

```bash
# Project A: Build pattern library
decy transpile-project --oracle --auto-fix --capture \
    ./project-a -o ./output-a

# Project B: Import patterns from A
decy transpile-project --oracle --import-patterns ./project-a.apr \
    ./project-b -o ./output-b
```

### Transferable Error Codes

Not all patterns transfer well between projects. The oracle only imports patterns for universal ownership/lifetime errors:

| Error Code | Description | Transferable |
|------------|-------------|--------------|
| E0382 | Borrow of moved value | Yes |
| E0499 | Multiple mutable borrows | Yes |
| E0506 | Cannot assign to borrowed | Yes |
| E0597 | Does not live long enough | Yes |
| E0515 | Cannot return reference to local | Yes |
| E0308 | Type mismatch | No (project-specific) |
| E0133 | Unsafe required | No (context-specific) |

## CI Integration

### JSON Output

```bash
decy transpile --oracle --oracle-report json input.c
```

Output:
```json
{
  "metrics": {
    "queries": 10,
    "hits": 8,
    "misses": 2,
    "fixes_applied": 8,
    "fixes_verified": 7
  },
  "hit_rate_pct": 80.0,
  "fix_success_rate_pct": 87.5,
  "passed": true,
  "thresholds": {
    "min_hit_rate": 0.5,
    "min_fix_rate": 0.8
  }
}
```

### Markdown Output

```bash
decy transpile --oracle --oracle-report markdown input.c > report.md
```

Output:
```markdown
## Oracle CI Report

| Metric | Value |
|--------|-------|
| Queries | 10 |
| Hits | 8 |
| Hit Rate | 80.0% |
| Fixes Applied | 8 |
| Fixes Verified | 7 |
| Fix Success Rate | 87.5% |

### Status: PASSED
```

### Prometheus Output

```bash
decy transpile --oracle --oracle-report prometheus input.c
```

Output:
```
# HELP decy_oracle_queries_total Total oracle queries
# TYPE decy_oracle_queries_total counter
decy_oracle_queries_total 10

# HELP decy_oracle_hit_rate Current hit rate
# TYPE decy_oracle_hit_rate gauge
decy_oracle_hit_rate 0.8
```

### GitHub Actions Example

```yaml
name: Transpile with Oracle

on: [push, pull_request]

jobs:
  transpile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install decy
        run: cargo install decy --features oracle

      - name: Transpile with oracle
        run: |
          decy transpile-project \
            --oracle \
            --auto-fix \
            --capture \
            --oracle-report markdown \
            ./src -o ./rust-src > oracle-report.md

      - name: Upload report
        uses: actions/upload-artifact@v4
        with:
          name: oracle-report
          path: oracle-report.md
```

## Metrics

The oracle tracks several metrics for observability:

| Metric | Description |
|--------|-------------|
| `queries` | Total oracle queries made |
| `hits` | Queries that returned suggestions |
| `misses` | Queries with no matching patterns |
| `fixes_applied` | Fixes that were applied |
| `fixes_verified` | Fixes that compiled successfully |
| `patterns_captured` | New patterns learned |
| `patterns_imported` | Patterns loaded from external .apr |

### Hit Rate

```
hit_rate = hits / queries
```

Target: >= 50% (improves as patterns accumulate)

### Fix Success Rate

```
fix_success_rate = fixes_verified / fixes_applied
```

Target: >= 80% (indicates pattern quality)

## Building with Oracle Support

The oracle requires the `oracle` feature flag:

```bash
# Build with oracle
cargo build --features oracle

# Run tests with oracle
cargo test --features oracle

# Install with oracle
cargo install decy --features oracle
```

## Pattern File Format

Patterns are stored in `.apr` files (Aprender binary format with zstd compression). These files are managed automatically by the oracle but can be:

- Copied between machines
- Shared via version control
- Imported from other PAIML transpilers (depyler, bashrs)

## Troubleshooting

### No patterns found

```
Oracle Statistics:
Queries: 10
Fixes applied: 0
```

**Solution**: Run with `--capture` to build initial pattern library, or import patterns from another project.

### Low hit rate

```
Hit Rate: 20%
```

**Solutions**:
1. Import patterns: `--import-patterns base.apr`
2. Run more transpilations with `--capture`
3. Lower threshold: `--oracle-threshold 0.5`

### Fixes not compiling

```
Fix Success Rate: 40%
```

**Solutions**:
1. Raise threshold: `--oracle-threshold 0.8`
2. Patterns may be project-specific; rebuild pattern library
3. Check for project-specific type definitions

## Related Documentation

- [CLI Reference](../reference/cli.md) - Complete command reference
- [entrenar CITL](https://github.com/paiml/entrenar) - Pattern storage system
- [Oracle Specification](../../docs/specifications/oracle-integration-spec.md) - Technical specification
