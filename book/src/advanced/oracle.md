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

## Oracle Training

The oracle learns from transpilation errors through a structured training pipeline. This section covers the CLI commands and workflow for training.

### Training CLI Commands

```bash
# Seed oracle with patterns from another project (cross-project transfer)
decy oracle seed --from ../depyler/depyler.apr

# Show oracle statistics
decy oracle stats
decy oracle stats --format json
decy oracle stats --format prometheus

# Retire obsolete patterns (Kaizen - continuous improvement)
decy oracle retire --dry-run
decy oracle retire --archive-path ./retired-patterns.apr

# Validate oracle on a corpus
decy oracle validate ./corpus/
```

### Training Workflow

The recommended training workflow follows Toyota Way principles:

#### Phase 1: Bootstrap (Yokoten)

**Option A: Cold Start Bootstrap** (Recommended for new installations)

Use the built-in bootstrap patterns for common C→Rust errors:

```bash
# Preview available bootstrap patterns
decy oracle bootstrap --dry-run

# Bootstrap the oracle with seed patterns
decy oracle bootstrap

# Verify patterns loaded
decy oracle stats
```

This loads 25+ predefined patterns for errors like:
- E0308 (type mismatch): pointer_to_reference, type_coercion
- E0133 (unsafe): unsafe_deref, unsafe_extern
- E0382 (use after move): clone_before_move, borrow_instead_of_move
- E0499 (multiple mutable borrows): sequential_mutable_borrow
- E0597/E0515 (lifetime): extend_lifetime, return_owned

**Option B: Cross-Project Import**

Seed the oracle with patterns from related projects:

```bash
# Import ownership/lifetime patterns from depyler (Python→Rust)
decy oracle seed --from ../depyler/depyler.apr

# Check import statistics
decy oracle stats
```

**Smart Import Filtering**: Not all patterns transfer between languages. The oracle's smart import filter:
- Accepts: `AddBorrow`, `AddLifetime` patterns (universal)
- Filters: Python-specific patterns (list cloning, etc.)
- Warns: Ambiguous patterns for manual review

#### Phase 2: Corpus Training (Genchi Genbutsu)

Train on real C code using the reprorusted-c-cli corpus:

```bash
# Clone training corpus
git clone https://github.com/paiml/reprorusted-c-cli ../reprorusted-c-cli

# Validate corpus diversity
decy oracle validate ../reprorusted-c-cli/coreutils/

# Train with pattern capture
decy transpile-project \
    --oracle \
    --auto-fix \
    --capture \
    ../reprorusted-c-cli/coreutils/ \
    -o ./output/
```

#### Phase 3: Validation (Jidoka)

Verify fix quality with semantic validation:

```bash
# Run validation on held-out corpus
decy oracle validate ./test-corpus/

# Check metrics
decy oracle stats --format markdown
```

**Semantic Verification**: Patterns must pass both:
1. **Compilation check**: `rustc` compiles without errors
2. **Test suite check**: Unit tests pass (when available)

Patterns that only compile get weight 0.6; fully verified patterns get weight 1.0.

#### Phase 4: Maintenance (Kaizen)

Retire obsolete patterns periodically:

```bash
# Preview retirements
decy oracle retire --dry-run

# Apply retirements
decy oracle retire --archive-path ./archive/retired-$(date +%Y%m%d).apr
```

**Retirement Policy**:
- Low usage: < 5 uses in 30 days
- High failure: < 30% success rate
- Superseded: Better pattern exists with > 20% improvement

### Corpus Diversity Validation

The oracle validates training corpus diversity using Jensen-Shannon divergence:

```bash
decy oracle validate ./corpus/
```

Output:
```
=== Corpus Diversity Analysis ===
Files: 19
Lines of code: 6180

C Construct Coverage:
  RawPointer: 45
  MallocFree: 23
  Struct: 18
  ForLoop: 67
  Switch: 12

=== Validation Results ===
Files processed: 19
Transpile success: 15
Transpile failed: 4
Success rate: 78.9%

Error Distribution:
  E0382: 3 (Ownership)
  E0597: 1 (Lifetime)

✅ Corpus diversity validation: PASSED
```

**Acceptance Criteria**: Jensen-Shannon divergence < 0.15 between training corpus and external validation corpora.

### Training Metrics

Monitor training progress with these metrics:

| Metric | Target | Description |
|--------|--------|-------------|
| Hit Rate | ≥ 50% | Queries returning suggestions |
| Fix Success Rate | ≥ 80% | Fixes that compile successfully |
| Full Verification Rate | ≥ 60% | Fixes passing tests |
| Pattern Count | Growing | Accumulated patterns |
| Retirement Rate | < 10% | Patterns retired per sweep |

### Example: Training on reprorusted-c-cli

Complete training workflow:

```bash
# 1. Clone corpus
git clone https://github.com/paiml/reprorusted-c-cli ../reprorusted-c-cli

# 2. Bootstrap oracle (cold start)
decy oracle bootstrap
decy oracle stats

# 3. Train on coreutils
for util in ../reprorusted-c-cli/coreutils/*/; do
    echo "Training on: $util"
    decy transpile-project \
        --oracle \
        --auto-fix \
        --capture \
        "$util" \
        -o "./trained-output/$(basename $util)/"
done

# 4. Check results
decy oracle stats --format markdown

# 5. Validate on held-out set
decy oracle validate ../reprorusted-c-cli/validation/

# 6. Retire low-quality patterns
decy oracle retire --dry-run
```

## AI-First Model Training

Decy's oracle supports generating training data for LLM fine-tuning. The goal is to train a model that "intuits" safe C-to-Rust transformations.

### Golden Traces

Golden Traces are verified C→Rust transformation pairs used as training data:

```rust
pub struct GoldenTrace {
    pub c_snippet: String,       // Input C code
    pub rust_snippet: String,    // Verified safe Rust output
    pub safety_explanation: String, // Chain-of-thought reasoning
    pub tier: TraceTier,         // P0/P1/P2 complexity
}
```

### Generating Training Data

```bash
# Generate Golden Traces from a C corpus
decy oracle generate-traces \
    --corpus ./c-corpus \
    --output ./traces.jsonl \
    --tier P0

# Preview without writing (dry run)
decy oracle generate-traces \
    --corpus ./c-corpus \
    --output ./traces.jsonl \
    --dry-run
```

**Training Tiers**:

| Tier | Complexity | Examples |
|------|------------|----------|
| P0 | Simple | Type casts, basic functions |
| P1 | Medium | File I/O, format strings |
| P2 | Complex | Ownership, lifetimes, concurrency |

### Querying Fix Patterns

Look up fix patterns for specific rustc error codes:

```bash
# Query patterns for type mismatch errors
decy oracle query --error E0308

# Query with context for better matches
decy oracle query --error E0382 --context "let x = value; use(x);"

# Get JSON output for tooling integration
decy oracle query --error E0308 --format json
```

**Supported Error Codes**:

| Code | Description | Pattern Count |
|------|-------------|---------------|
| E0308 | Type mismatch | 12+ patterns |
| E0133 | Unsafe required | 3+ patterns |
| E0382 | Use after move | 5+ patterns |
| E0499 | Multiple mutable borrows | 3+ patterns |
| E0597 | Lifetime issues | 4+ patterns |

### Export to HuggingFace

Export patterns in ML-ready formats:

```bash
# JSONL format (default)
decy oracle export ./patterns.jsonl --format jsonl

# ChatML format for chat fine-tuning
decy oracle export ./patterns.chatml --format chatml

# Alpaca format
decy oracle export ./patterns.alpaca --format alpaca

# Generate dataset card
decy oracle export ./patterns.jsonl --format jsonl --with-card
```

### Training Workflow

Complete workflow for generating model training data:

```bash
# 1. Bootstrap with seed patterns
decy oracle bootstrap

# 2. Generate traces from corpus
decy oracle generate-traces \
    --corpus ./reprorusted-c-cli \
    --output ./golden-traces.jsonl \
    --tier P0

# 3. Export to HuggingFace format
decy oracle export ./dataset.jsonl --format jsonl --with-card

# 4. View statistics
decy oracle stats --format markdown
```

## Related Documentation

- [CLI Reference](../reference/cli.md) - Complete command reference
- [entrenar CITL](https://github.com/paiml/entrenar) - Pattern storage system
- [Unified Specification](../../docs/specifications/decy-unified-spec.md) - Full technical specification
