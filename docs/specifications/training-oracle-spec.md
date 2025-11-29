# Oracle Training Specification for Decy

**Version**: 1.1.0
**Status**: Approved with Comments
**Reviewed By**: Gemini (CLI Agent)
**Author**: Claude Code / PAIML Team
**Date**: 2025-11-29
**Related**: [oracle-integration-spec.md](oracle-integration-spec.md), [reprorusted-c-cli](https://github.com/paiml/reprorusted-c-cli)

---

## Review & Standardization Principles (Toyota Way)

This specification implements the **Toyota Way** principles:

* **Genchi Genbutsu (Go and See)**: Training on real GNU coreutils code, not synthetic examples [6]
* **Yokoten (Horizontal Deployment)**: Cross-project pattern transfer from depyler [1]
* **Jidoka (Automation with Human Intelligence)**: Compiler feedback as automatic oracle [9]
* **Kaizen (Continuous Improvement)**: Iterative training with curriculum learning [3]

---

## Abstract

This specification defines the training pipeline for decy's CITL (Compiler-in-the-Loop) oracle using the `reprorusted-c-cli` corpus. The oracle learns fix patterns from real C→Rust transpilation errors to achieve **cost-free steady-state operation** by replacing LLM queries with pattern-based retrieval [1]. Building on research in automated program repair [2, 5] and retrieval-augmented generation [1], we implement a multi-phase training approach with curriculum learning [3] and active learning [4] to maximize pattern coverage while minimizing training cost.

## 1. Motivation

### 1.1 Current State

The decy-oracle infrastructure is **complete but empty**:

| Component | Status | Location |
|-----------|--------|----------|
| DecyOracle struct | ✅ Implemented | `crates/decy-oracle/src/oracle.rs` |
| CLI integration | ✅ Implemented | `crates/decy/src/oracle_integration.rs` |
| Pattern store (APR) | ✅ Implemented | Via entrenar CITL |
| Cross-project transfer | ✅ Implemented | `--import-patterns` flag |
| CI metrics export | ✅ Implemented | JSON/Markdown/Prometheus |
| **Training corpus** | ❌ Empty | `reprorusted-c-cli/` |
| **Learned patterns** | ❌ None | `*.apr` files |

### 1.2 C→Rust Error Distribution

Analysis of the `reprorusted-c-cli` corpus (19 coreutils, 47 functions, 6180 LOC) reveals:

```
Expected Error Distribution (from corpus_metadata.yaml):
├─ E0506 (Cannot assign to borrowed)     - 412 occurrences (31%)
├─ E0499 (Multiple mutable borrows)      - 278 occurrences (21%)
├─ E0382 (Use after move)                - 213 occurrences (16%)
├─ E0308 (Type mismatch)                 - 133 occurrences (10%)
├─ E0133 (Unsafe required)               -  93 occurrences (7%)
├─ E0597 (Does not live long enough)     -  80 occurrences (6%)
├─ E0515 (Cannot return reference)       -  66 occurrences (5%)
└─ Other                                 -  53 occurrences (4%)
```

**Key insight**: 74% of errors are ownership/lifetime-related, validating our focus on `decy-ownership` crate [6].

### 1.3 Cross-Project Transfer Opportunity

Per the *Yokoten* principle, borrow-checker fixes are largely **language-agnostic** on the source side [1]:

| Error Code | depyler Compatible | C-Specific |
|------------|-------------------|------------|
| E0382 (Use after move) | ✅ | |
| E0499 (Multiple mutable borrows) | ✅ | |
| E0506 (Cannot assign to borrowed) | ✅ | |
| E0597 (Does not live long enough) | ✅ | |
| E0515 (Cannot return reference) | ✅ | |
| E0133 (Unsafe required) | | ✅ C raw pointers |
| E0308 (Type mismatch) | | ✅ C type system |

**Estimated bootstrap**: 62% of patterns can be seeded from depyler, reducing cold-start time.

**Risk (per Gemini review)**: The root cause of ownership errors differs between Python (implicit reference counting) and C (pointer aliasing, manual memory management). A "smart import" filter is required—see Section 3.1.2.

### 1.4 Corpus Diversity Validation (Genchi Genbutsu)

**Reviewer Comment**: Ensure the reprorusted-c-cli error distribution matches broader C code, not just GNU coding styles.

**Action**: Compare error histogram against unannotated C codebases:

| Corpus | LOC | Primary Patterns | Validation Status |
|--------|-----|------------------|-------------------|
| reprorusted-c-cli | 6,180 | GNU coreutils style | Primary training |
| Redis (sample) | ~5,000 | Event-loop, callbacks | Diversity check |
| SQLite (sample) | ~5,000 | Single-file, heavy macros | Diversity check |

```bash
# Corpus diversity validation
decy analyze --error-histogram ../redis/src/*.c > redis_errors.json
decy analyze --error-histogram ../sqlite/src/*.c > sqlite_errors.json

# Compare distributions
python scripts/compare_distributions.py \
    reprorusted_errors.json \
    redis_errors.json \
    sqlite_errors.json \
    --threshold 0.85  # Jensen-Shannon divergence
```

**Acceptance Criterion**: Jensen-Shannon divergence < 0.15 between reprorusted-c-cli and external corpora.

### 1.5 Cost Model

| Scenario | Per-Error Cost | 1000 Errors (corpus scale) | Annual (10K/day) |
|----------|---------------|---------------------------|------------------|
| LLM-only | $0.05/error | $50/run | $182K |
| Oracle @ 80% hit | $0.01/error | $10/run | $36K |
| **Savings** | | $40/run | **$146K** |

## 2. Architecture

### 2.1 Training Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ORACLE TRAINING PIPELINE                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Phase 1: BOOTSTRAP                                                         │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐      │
│  │  depyler.apr     │───▶│  import_patterns │───▶│  decy.apr        │      │
│  │  (5+ error codes)│    │  (cross-project) │    │  (seeded)        │      │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘      │
│                                                                             │
│  Phase 2: CORPUS TRAINING                                                   │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐      │
│  │  reprorusted-c   │───▶│  decy transpile  │───▶│  rustc compile   │      │
│  │  (19 coreutils)  │    │  --capture       │    │  (feedback)      │      │
│  └──────────────────┘    └──────────────────┘    └────────┬─────────┘      │
│                                                            │                │
│                          ┌────────────────────────────────┘                │
│                          ▼                                                  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    CITL FEEDBACK LOOP [9]                             │  │
│  │  for each error:                                                      │  │
│  │    1. Query oracle for fix suggestion                                 │  │
│  │    2. If found: Apply fix, verify with rustc                         │  │
│  │    3. If verified: Record pattern to .apr (success_count++)          │  │
│  │    4. If not found: Flag for manual review or LLM assist             │  │
│  │    5. Capture new pattern if fix discovered                          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  Phase 3: CURRICULUM LEARNING [3]                                          │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐      │
│  │  Tier 1: P0      │───▶│  Tier 2: P1      │───▶│  Tier 3: P2      │      │
│  │  (yes,true,echo) │    │  (cp,mv,rm,ls)   │    │  (sort,chmod)    │      │
│  │  Simple patterns │    │  I/O patterns    │    │  Complex ptr ops │      │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘      │
│                                                                             │
│  Phase 4: ACTIVE LEARNING [4]                                              │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │  Query strategy: UncertaintySampling + EntropySampling                │  │
│  │  - Prioritize errors with low oracle confidence                       │  │
│  │  - Focus human review on highest-value patterns                       │  │
│  │  - Maximize coverage per human annotation                             │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  Phase 5: VALIDATION & EXPORT                                              │
│  ┌──────────────────┐    ┌──────────────────┐    ┌──────────────────┐      │
│  │  decy.apr        │───▶│  Mutation test   │───▶│  CI/CD deploy    │      │
│  │  (trained)       │    │  (kill rate)     │    │  (metrics)       │      │
│  └──────────────────┘    └──────────────────┘    └──────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Component Integration

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PAIML ECOSYSTEM INTEGRATION                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐                                                           │
│  │  entrenar   │  CITL Core: DecisionPatternStore, DecisionCITL            │
│  │  (training) │  - Tarantula fault localization [8]                       │
│  │             │  - RAG retrieval with RRF fusion [1, 7]                   │
│  └──────┬──────┘  - .apr persistence with zstd compression                 │
│         │                                                                   │
│  ┌──────▼──────┐                                                           │
│  │  aprender   │  ML Primitives: ActiveLearning, CurriculumScheduler       │
│  │  (ML ops)   │  - UncertaintySampling, EntropySampling [4]               │
│  │             │  - Difficulty tiers for progressive training [3]          │
│  └──────┬──────┘  - ErrorEncoder with GNN for pattern matching [10]        │
│         │                                                                   │
│  ┌──────▼──────┐                                                           │
│  │  trueno-rag │  Hybrid Retrieval: BM25 + Dense + RRF                     │
│  │  (retrieval)│  - FusionStrategy::RRF with k=60 [7]                      │
│  │             │  - ChunkId for pattern indexing                           │
│  └──────┬──────┘                                                           │
│         │                                                                   │
│  ┌──────▼──────┐                                                           │
│  │ decy-oracle │  Integration Layer: DecyOracle, OracleConfig              │
│  │             │  - C-specific decision categories                         │
│  │             │  - Cross-project pattern import                           │
│  └─────────────┘                                                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 3. Implementation Plan

### Phase 1: Bootstrap from depyler (Day 1)

**Goal**: Seed oracle with cross-project patterns for immediate value.

#### 3.1.1 Import depyler Patterns

```bash
# In reprorusted-c-cli/
make citl-seed
```

**Implementation**:

```rust
// crates/decy/src/commands/oracle.rs

pub fn seed_from_depyler(depyler_apr: &Path) -> Result<usize> {
    let mut oracle = DecyOracle::new(OracleConfig::default())?;

    // Import only compatible error codes
    let compatible = ["E0382", "E0499", "E0506", "E0597", "E0515"];
    let count = oracle.import_patterns_filtered(depyler_apr, &compatible)?;

    oracle.save()?;
    println!("Imported {count} patterns from depyler");
    Ok(count)
}
```

**Expected Outcome**:
- Import ~50-100 patterns for 5 compatible error codes
- Immediate 30-40% hit rate on ownership errors

#### 3.1.2 Smart Import Filter (Yokoten Enhancement)

**Reviewer Comment**: Don't blindly bulk import—verify fix strategies are applicable to C context, not just Python patterns.

**Problem**: Python ownership issues stem from implicit reference counting; C issues stem from pointer aliasing and manual memory management. Same error code, different root causes.

```rust
// crates/decy-oracle/src/import.rs

/// Filter patterns by fix strategy applicability, not just error code
pub fn smart_import_filter(pattern: &FixPattern) -> ImportDecision {
    // Check if fix strategy is C-applicable
    match analyze_fix_strategy(&pattern.fix_diff) {
        FixStrategy::AddClone => {
            // Clone semantics differ: Python shallow copy vs Rust deep clone
            // Only import if pattern doesn't rely on Python __copy__ semantics
            if pattern.metadata.get("source_construct") == Some(&"list_copy".into()) {
                ImportDecision::Reject("Python list copy != Rust clone")
            } else {
                ImportDecision::Accept
            }
        }
        FixStrategy::AddBorrow => {
            // Borrow semantics are universal - safe to import
            ImportDecision::Accept
        }
        FixStrategy::AddLifetime => {
            // Lifetime patterns transfer well
            ImportDecision::Accept
        }
        FixStrategy::WrapInOption => {
            // Python None vs C NULL have different semantics
            // Verify pattern handles C NULL pointer checks
            if pattern.fix_diff.contains("NULL") || pattern.fix_diff.contains("nullptr") {
                ImportDecision::Accept
            } else {
                ImportDecision::AcceptWithWarning("Verify NULL handling")
            }
        }
        FixStrategy::Unknown => ImportDecision::Reject("Unknown strategy"),
    }
}

pub enum ImportDecision {
    Accept,
    AcceptWithWarning(&'static str),
    Reject(&'static str),
}
```

**Validation Metrics**:

| Strategy | Import Rate | Reason |
|----------|-------------|--------|
| AddBorrow | 95% | Universal semantics |
| AddLifetime | 90% | Universal semantics |
| AddClone | 60% | Python-specific cases filtered |
| WrapInOption | 70% | NULL semantics differ |

#### 3.1.3 Verify Bootstrap Quality

```bash
# Test on a simple example
decy transpile examples/coreutils_yes/original.c --oracle --dry-run

# Expected output:
# Oracle queries: 3
# Cache hits: 2 (67%)
# Patterns from: depyler-seed
```

### Phase 2: Corpus Training (Days 2-5)

**Goal**: Train on full corpus with CITL feedback loop.

#### 3.2.1 Tier 1: P0 Utilities (Day 2)

Simple utilities with minimal complexity:

| Utility | LOC | Functions | Expected Errors |
|---------|-----|-----------|-----------------|
| yes | 30 | 1 | 2 |
| true | 10 | 1 | 0 |
| false | 10 | 1 | 0 |
| echo | 80 | 2 | 5 |
| cat | 150 | 4 | 8 |
| wc | 120 | 3 | 6 |
| head | 100 | 2 | 4 |
| tail | 140 | 3 | 7 |

```bash
# Train on P0 tier
cd ../reprorusted-c-cli
make citl-train TIER=P0

# This runs:
for util in yes true false echo cat wc head tail; do
    decy transpile "examples/coreutils_$util/original.c" \
        --oracle \
        --capture-patterns \
        --max-retries 5
done
```

**Pattern Capture Logic** (from oracle_integration.rs):

```rust
// When a fix is discovered and verified:
if compilation_success {
    for error in &pending_verified {
        oracle.record_fix_verified(error);  // Increments success_count
        result.patterns_captured += 1;
    }
    oracle.save()?;  // Persist to .apr
}
```

#### 3.2.1.1 Semantic Verification (Jidoka Enhancement)

**Reviewer Comment**: Compilation alone is insufficient. A patch might fix the borrow error but change program behavior. Patterns must pass both `rustc` AND unit tests.

**Implementation**:

```rust
// crates/decy/src/oracle_integration.rs

/// Enhanced verification: compile + test
pub fn verify_fix_semantically(
    rust_code: &str,
    original_c: &Path,
    test_suite: Option<&Path>,
) -> VerificationResult {
    // Step 1: Syntactic verification (existing)
    if let Err(e) = check_rust_compilation(rust_code) {
        return VerificationResult::CompileFailed(e);
    }

    // Step 2: Semantic verification (NEW - Jidoka)
    if let Some(tests) = test_suite {
        // Run original C test suite against transpiled Rust
        match run_test_suite(tests, rust_code) {
            TestResult::AllPassed => VerificationResult::FullyVerified,
            TestResult::SomeFailed(failures) => {
                VerificationResult::BehaviorChanged(failures)
            }
            TestResult::NoTests => VerificationResult::CompilesOnly,
        }
    } else {
        VerificationResult::CompilesOnly
    }
}

pub enum VerificationResult {
    FullyVerified,           // Compiles + all tests pass
    CompilesOnly,            // Compiles but no tests available
    BehaviorChanged(Vec<String>), // Compiles but tests fail
    CompileFailed(String),   // Doesn't compile
}
```

**Pattern Promotion Rules**:

| Verification Level | Pattern Action | Confidence |
|-------------------|----------------|------------|
| FullyVerified | Promote to oracle, high weight | 1.0 |
| CompilesOnly | Promote with warning, low weight | 0.6 |
| BehaviorChanged | Reject, flag for review | 0.0 |
| CompileFailed | Reject | 0.0 |

**Test Suite Integration** (coreutils):

```bash
# Each coreutil has existing tests
examples/coreutils_cat/
├── original.c
├── metadata.yaml
└── tests/           # Existing C test cases
    ├── test_basic.sh
    ├── test_flags.sh
    └── test_edge_cases.sh

# Decy runs these against transpiled Rust:
decy transpile original.c -o cat.rs --verify-tests tests/
```

#### 3.2.2 Tier 2: P1 Utilities (Days 3-4)

I/O-heavy utilities with buffer management:

| Utility | LOC | Functions | Expected Errors |
|---------|-----|-----------|-----------------|
| cp | 600 | 8 | 45 |
| mv | 400 | 5 | 30 |
| rm | 350 | 4 | 25 |
| ls | 800 | 10 | 55 |
| mkdir | 200 | 3 | 15 |
| ln | 250 | 4 | 18 |

```bash
make citl-train TIER=P1
```

**Curriculum Progression** [3]:

```rust
// Per Bengio et al. (2009), curriculum learning accelerates convergence
pub struct CurriculumScheduler {
    current_tier: Tier,
    accuracy_threshold: f32,  // Advance when accuracy >= threshold
}

impl CurriculumScheduler {
    pub fn should_advance(&self, metrics: &TrainingMetrics) -> bool {
        metrics.accuracy >= self.accuracy_threshold  // Default: 0.7
    }

    pub fn advance(&mut self) -> Option<Tier> {
        match self.current_tier {
            Tier::P0 => { self.current_tier = Tier::P1; Some(Tier::P1) }
            Tier::P1 => { self.current_tier = Tier::P2; Some(Tier::P2) }
            Tier::P2 => None  // Training complete
        }
    }
}
```

#### 3.2.3 Tier 3: P2 Utilities (Day 5)

Complex utilities with pointer arithmetic:

| Utility | LOC | Functions | Expected Errors |
|---------|-----|-----------|-----------------|
| sort | 500 | 6 | 40 |
| uniq | 250 | 4 | 20 |
| chmod | 300 | 4 | 22 |
| chown | 280 | 4 | 20 |
| cut | 320 | 5 | 25 |

```bash
make citl-train TIER=P2
```

### Phase 3: Active Learning (Days 6-7)

**Goal**: Maximize pattern coverage with minimal human intervention.

#### 3.3.1 Uncertainty Sampling [4]

Query patterns where oracle confidence is lowest:

```rust
// From aprender/src/active_learning.rs
pub struct UncertaintySampling;

impl QueryStrategy for UncertaintySampling {
    fn score(&self, predictions: &[Vector<f32>]) -> Vec<f32> {
        predictions.iter()
            .map(|p| {
                let max_prob = p.as_slice().iter().fold(0.0_f32, |a, &b| a.max(b));
                1.0 - max_prob  // Lower confidence = higher priority
            })
            .collect()
    }
}
```

**Active Learning Loop**:

```rust
// Prioritize errors for human review
let uncertain_errors = oracle.get_low_confidence_errors(threshold: 0.5);
for error in uncertain_errors.take(10) {
    println!("Review needed: {} - {}", error.code, error.message);
    println!("  Context: {}", error.context);

    // Human provides fix, system captures pattern
    if let Some(fix) = prompt_human_fix(&error) {
        oracle.capture_pattern(&error, &fix);
    }
}
```

#### 3.3.2 Entropy-Based Prioritization

Focus on errors with highest information gain:

```rust
// From aprender/src/active_learning.rs
pub struct EntropySampling;

impl QueryStrategy for EntropySampling {
    fn score(&self, predictions: &[Vector<f32>]) -> Vec<f32> {
        predictions.iter()
            .map(|p| {
                let mut entropy = 0.0;
                for &prob in p.as_slice() {
                    if prob > 1e-10 {
                        entropy -= prob * prob.ln();
                    }
                }
                entropy  // Higher entropy = more uncertain
            })
            .collect()
    }
}
```

#### 3.3.3 Pattern Retirement Policy (Kaizen Enhancement)

**Reviewer Comment**: As the oracle improves (Kaizen), older patterns may become obsolete. Add a mechanism to prune patterns that are rarely used or frequently superseded.

**Retirement Criteria**:

| Condition | Action | Threshold |
|-----------|--------|-----------|
| Low usage | Retire | < 5 uses in 30 days |
| High failure rate | Retire | success_rate < 0.3 |
| Superseded | Retire | Better pattern exists (higher success_rate for same error) |
| Deprecated strategy | Retire | Manual flag by maintainer |

**Implementation**:

```rust
// crates/decy-oracle/src/retirement.rs

pub struct PatternRetirementPolicy {
    min_usage_threshold: usize,        // Minimum uses to keep
    min_success_rate: f32,             // Minimum success rate
    evaluation_window_days: u32,       // Window for usage tracking
}

impl PatternRetirementPolicy {
    pub fn evaluate(&self, pattern: &FixPattern, stats: &PatternStats) -> RetirementDecision {
        // Criterion 1: Low usage
        if stats.uses_in_window < self.min_usage_threshold {
            return RetirementDecision::Retire(RetirementReason::LowUsage);
        }

        // Criterion 2: High failure rate
        if pattern.success_rate() < self.min_success_rate {
            return RetirementDecision::Retire(RetirementReason::HighFailureRate);
        }

        // Criterion 3: Superseded by better pattern
        if let Some(better) = stats.better_pattern_exists {
            if better.success_rate() > pattern.success_rate() + 0.1 {
                return RetirementDecision::Retire(RetirementReason::Superseded);
            }
        }

        RetirementDecision::Keep
    }
}

pub enum RetirementDecision {
    Keep,
    Retire(RetirementReason),
    Archive,  // Keep for historical analysis but don't use
}

pub enum RetirementReason {
    LowUsage,
    HighFailureRate,
    Superseded,
    ManualDeprecation,
}
```

**CLI Command**:

```bash
# Run retirement sweep
decy oracle retire --dry-run          # Preview retirements
decy oracle retire --execute          # Actually retire patterns
decy oracle retire --archive-path ./retired_patterns.apr

# Example output:
# Evaluating 247 patterns...
# - Retiring 12 patterns (low usage)
# - Retiring 5 patterns (high failure rate)
# - Retiring 8 patterns (superseded)
# - Keeping 222 patterns
#
# Run with --execute to apply retirements
```

**Scheduled Maintenance**:

```yaml
# .github/workflows/oracle-maintenance.yml
name: Oracle Pattern Maintenance
on:
  schedule:
    - cron: '0 2 * * 0'  # Weekly on Sunday 2am

jobs:
  retire-patterns:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Retire obsolete patterns
        run: decy oracle retire --execute --archive-path ./archived/
      - name: Report metrics
        run: decy oracle stats --format markdown >> $GITHUB_STEP_SUMMARY
```

### Phase 4: Validation (Day 8)

**Goal**: Verify oracle quality meets production standards.

#### 3.4.1 Accuracy Metrics

```bash
# Run validation suite
make citl-validate

# Expected output:
# Oracle Accuracy: 82% (target: 80%)
# Pattern Coverage: 7/7 error codes
# Cross-Project Transfer: 62% patterns from depyler
# Novel Patterns: 38% unique to C→Rust
```

#### 3.4.2 Mutation Testing

Verify patterns are robust against code variations:

```rust
// Test pattern generalization
#[test]
fn test_pattern_generalizes() {
    let oracle = DecyOracle::load("decy.apr")?;

    // Original error context
    let original = "let x: &mut Vec<i32> = ...";

    // Mutated variations
    let variants = [
        "let y: &mut Vec<String> = ...",
        "let data: &mut Vec<u8> = ...",
    ];

    for variant in variants {
        let suggestion = oracle.suggest_fix(&error, &variant);
        assert!(suggestion.is_some(), "Pattern should generalize to: {}", variant);
    }
}
```

### Phase 5: CI/CD Integration (Day 9)

**Goal**: Deploy trained oracle to production.

#### 3.5.1 Export Metrics

```bash
# Generate CI report
decy oracle stats --format prometheus > metrics.prom
decy oracle stats --format json > metrics.json
decy oracle stats --format markdown > ORACLE_REPORT.md
```

**Sample Report**:

```markdown
# Decy Oracle Training Report

## Summary
- **Training Date**: 2025-11-29
- **Corpus**: reprorusted-c-cli v1.0.0
- **Total Patterns**: 247
- **Accuracy**: 82%

## Error Code Coverage

| Code | Patterns | Success Rate | Source |
|------|----------|--------------|--------|
| E0506 | 78 | 85% | 40% depyler, 60% decy |
| E0499 | 52 | 88% | 45% depyler, 55% decy |
| E0382 | 41 | 82% | 50% depyler, 50% decy |
| E0308 | 32 | 75% | 100% decy (C-specific) |
| E0133 | 18 | 70% | 100% decy (unsafe) |
| E0597 | 15 | 80% | 60% depyler, 40% decy |
| E0515 | 11 | 78% | 55% depyler, 45% decy |
```

#### 3.5.2 Deploy to Production

```bash
# Copy trained oracle to deployment location
cp decy.apr ~/.decy/patterns/

# Verify deployment
decy oracle status
# Output:
# Oracle: loaded (247 patterns)
# Last updated: 2025-11-29
# Accuracy: 82%
```

## 4. CLI Commands

### 4.1 New Oracle Training Commands

```bash
# Seed from depyler patterns
decy oracle seed --from ~/.depyler/patterns.apr

# Train on corpus
decy oracle train --corpus ../reprorusted-c-cli --tier P0

# Train with curriculum learning
decy oracle train --corpus ../reprorusted-c-cli --curriculum

# Active learning mode
decy oracle train --corpus ../reprorusted-c-cli --active-learning

# Validate trained oracle
decy oracle validate --corpus ../reprorusted-c-cli

# Export metrics
decy oracle stats --format json

# Show coverage report
decy oracle coverage
```

### 4.2 Makefile Targets (reprorusted-c-cli)

```makefile
# Bootstrap from depyler
citl-seed:
	@echo "Seeding oracle from depyler..."
	decy oracle seed --from ~/.depyler/patterns.apr
	@echo "Seeded patterns"

# Train on specific tier
citl-train:
	@echo "Training on tier $(TIER)..."
	@for util in $(TIER_$(TIER)_UTILS); do \
		decy transpile "examples/$$util/original.c" \
			--oracle --capture-patterns --max-retries 5; \
	done
	@echo "Training complete"

# Full training cycle
citl-improve: citl-seed
	$(MAKE) citl-train TIER=P0
	$(MAKE) citl-train TIER=P1
	$(MAKE) citl-train TIER=P2
	$(MAKE) citl-validate

# Validate oracle
citl-validate:
	decy oracle validate --corpus .
	decy oracle stats --format markdown > ORACLE_REPORT.md

# Export for CI
citl-export:
	decy oracle stats --format prometheus > metrics.prom
	cp ~/.decy/patterns/decy.apr ./artifacts/
```

## 5. Success Criteria

### 5.1 Quantitative Targets

| Metric | Target | Baseline | Measurement |
|--------|--------|----------|-------------|
| Oracle Accuracy | ≥80% | 0% | `decy oracle validate` |
| Pattern Count | ≥200 | 0 | `decy oracle stats` |
| Error Code Coverage | 7/7 | 0/7 | Unique codes in .apr |
| Cross-Project Transfer | ≥50% | 0% | Patterns from depyler |
| Training Time | <4 hours | N/A | Wall clock |
| Cold-Start Hit Rate | ≥30% | 0% | After seed only |

### 5.2 Qualitative Criteria

- [ ] All P0 utilities transpile with ≥70% fix rate
- [ ] All P1 utilities transpile with ≥60% fix rate
- [ ] All P2 utilities transpile with ≥50% fix rate
- [ ] No regression in existing decy functionality
- [ ] CI metrics export working (JSON, Markdown, Prometheus)
- [ ] Documentation updated with training instructions

## 6. Risk Mitigation

### 6.1 Identified Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Corpus too complex | Medium | High | Curriculum learning (P0→P1→P2) |
| Pattern overfitting | Medium | Medium | Cross-validation, mutation testing |
| depyler incompatibility | Low | Medium | Filter by error code compatibility |
| Training time exceeds estimate | Medium | Low | Parallel processing with rayon |
| Low hit rate | Medium | High | Active learning for edge cases |

### 6.2 Rollback Procedure

```bash
# If oracle quality degrades:
cp ~/.decy/patterns/decy.apr.backup ~/.decy/patterns/decy.apr
decy oracle reload

# If complete reset needed:
rm ~/.decy/patterns/decy.apr
decy oracle seed --from ~/.depyler/patterns.apr
```

## 7. Academic References

### Program Repair & Code Generation

1. **Lewis, P., Perez, E., Piktus, A., et al.** (2020). "Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks." *NeurIPS*, 33, 9459-9474. doi:10.48550/arXiv.2005.11401
   - *Foundation for RAG-based pattern retrieval in oracle*

2. **Le Goues, C., Nguyen, T., Forrest, S., & Weimer, W.** (2012). "GenProg: A Generic Method for Automatic Software Repair." *IEEE TSE*, 38(1), 54-72. doi:10.1109/TSE.2011.104
   - *Evolutionary approach to automated program repair*

3. **Bengio, Y., Louradour, J., Collobert, R., & Weston, J.** (2009). "Curriculum Learning." *ICML*, 41-48. doi:10.1145/1553374.1553380
   - *Theoretical foundation for P0→P1→P2 progressive training*

4. **Settles, B.** (2009). "Active Learning Literature Survey." *Computer Sciences Technical Report 1648*, University of Wisconsin-Madison.
   - *Query strategies for efficient pattern discovery*

5. **Long, F. & Rinard, M.** (2016). "Automatic Patch Generation by Learning Correct Code." *POPL*, 298-312. doi:10.1145/2837614.2837617
   - *Learning-based approach to patch generation*

### C→Rust & Ownership

6. **Emre, M., Schroeder, R., Dewey, K., & Hardekopf, B.** (2021). "Translating C to Safer Rust." *OOPSLA*, 121:1-121:29. doi:10.1145/3485498
   - *Domain-specific challenges in C→Rust transpilation*

### Retrieval & Ranking

7. **Cormack, G.V., Clarke, C.L.A., & Buettcher, S.** (2009). "Reciprocal Rank Fusion Outperforms Condorcet and Individual Rank Learning Methods." *SIGIR*, 758-759. doi:10.1145/1571941.1572114
   - *RRF algorithm used in trueno-rag for hybrid retrieval*

### Fault Localization

8. **Jones, J.A. & Harrold, M.J.** (2005). "Empirical Evaluation of the Tarantula Automatic Fault-Localization Technique." *ASE*, 273-282. doi:10.1145/1101908.1101949
   - *Statistical debugging technique used in entrenar CITL*

### Compiler Feedback Learning

9. **Wang, B., et al.** (2022). "Compilable Neural Code Generation with Compiler Feedback." *ACL*, 1853-1867. doi:10.18653/v1/2022.acl-long.130
   - *CITL architecture inspiration*

10. **Dou, S., et al.** (2024). "StepCoder: Improve Code Generation with Reinforcement Learning from Compiler Feedback." *arXiv:2402.01391*
    - *RL optimization of compiler feedback loops, curriculum learning acceleration*

### Additional Citations (Gemini Review)

11. **Gupta, R., Pal, S., Kanade, A., & Shevade, S.** (2017). "DeepFix: Fixing Common C Language Errors by Deep Learning." *AAAI Conference on Artificial Intelligence*.
    - *Supports: Iterative compiler feedback sufficient for neural program repair*

12. **Parvez, M.R., Ahmad, W., Chakraborty, S., Ray, B., & Chang, K.W.** (2021). "Retrieval Augmented Code Generation and Summarization." *Findings of EMNLP*.
    - *Supports: trueno-rag and pattern retrieval over pure generation*

13. **Nashid, N., Sintaha, M., & Mesbah, A.** (2023). "Retrieval-Based Prompt Selection for Code-Related Few-Shot Learning." *ICSE*.
    - *Supports: Retrieving specific patterns to prompt fix generation*

14. **Soviany, P., Ionescu, R.T., Rota, P., & Sebe, N.** (2022). "Curriculum Learning: A Survey." *International Journal of Computer Vision*.
    - *Supports: P0/P1/P2 tiering strategy for accelerated convergence*

15. **Liu, C., He, X., Qing, Y., & Sun, H.** (2021). "Multi-Task Learning for Cross-Language Source Code Tasks." *IEEE Access*.
    - *Supports: Cross-project pattern transfer (Python→C context)*

16. **Dor, B.S., & Elish, K.O.** (2020). "Active Learning for Software Engineering: A Systematic Mapping Study." *IEEE Access*.
    - *Supports: Uncertainty/entropy sampling for human review optimization*

17. **Qin, B., Chen, Y., Yu, Z., Song, L., & Zhang, Y.** (2020). "Understanding Memory and Thread Safety Practices and Issues in Real-World Rust Programs." *PLDI*.
    - *Supports: Focus on ownership/lifetime errors (E0382, E0506)*

18. **Gazzola, L., Micucci, D., & Mariani, L.** (2019). "Automatic Software Repair: A Survey." *IEEE Transactions on Software Engineering*.
    - *Supports: APR workflow: Fault Localization → Patch Generation → Verification*

19. **Yasunaga, M., & Liang, P.** (2020). "Graph-based Self-Supervised Program Repair from Diagnostic Feedback." *ICML*.
    - *Supports: GNN-based error encoding for pattern matching*

20. **Poppendieck, M., & Poppendieck, T.** (2003). *Lean Software Development: An Agile Toolkit.* Addison-Wesley.
    - *Supports: Toyota Way principles (Genchi Genbutsu, Jidoka) in software*

## 8. Related Work in PAIML Stack

### 8.1 entrenar CITL Module

- `DecisionPatternStore`: Hybrid BM25 + dense retrieval with RRF
- `DecisionCITL`: Tarantula fault localization
- `.apr` persistence format with zstd compression
- See: `entrenar/src/citl/`

### 8.2 aprender CITL Module

- `ErrorEncoder`: GNN-based error embedding
- `PatternLibrary`: HNSW index for pattern matching
- Curriculum learning with difficulty tiers
- Active learning strategies
- See: `aprender/src/citl/`

### 8.3 depyler Oracle

- First implementation of oracle query loop
- Proven on Python→Rust transpilation
- 84% baseline accuracy
- See: `depyler/docs/specifications/metaheuristic-oracle-phase2-spec.md`

### 8.4 reprorusted-c-cli Corpus

- 19 GNU coreutils examples
- 47 functions, 6180 LOC
- Pre-annotated with expected errors
- Tiered by complexity (P0/P1/P2)
- See: `reprorusted-c-cli/corpus_metadata.yaml`

## 9. Implementation Timeline

| Day | Phase | Activities | Deliverables |
|-----|-------|------------|--------------|
| 0 | Pre-flight | Corpus diversity check (Redis, SQLite) | Diversity report |
| 1 | Bootstrap | Smart import from depyler, verify integration | `decy.apr` (seeded) |
| 2 | Training P0 | Train on yes, true, false, echo, cat, wc, head, tail | +32 patterns |
| 3-4 | Training P1 | Train on cp, mv, rm, ls, mkdir, ln + semantic verification | +120 patterns |
| 5 | Training P2 | Train on sort, uniq, chmod, chown, cut | +75 patterns |
| 6-7 | Active Learning | Review uncertain patterns, capture edge cases | +20 patterns |
| 8 | Validation | Accuracy testing, mutation testing, coverage | Test report |
| 9 | Deployment | CI integration, metrics export, documentation | Production oracle |
| Ongoing | Maintenance | Weekly pattern retirement sweep | Lean oracle |

---

*Specification created: 2025-11-29*
*Reviewed: 2025-11-29 by Gemini (CLI Agent)*
*Status: IMPLEMENTED - All 5 tickets complete (DECY-101 through DECY-105)*

**Gemini Review Incorporations**:
1. ✅ Section 1.4: Corpus Diversity Validation (Genchi Genbutsu)
2. ✅ Section 3.1.2: Smart Import Filter (Yokoten Enhancement)
3. ✅ Section 3.2.1.1: Semantic Verification (Jidoka Enhancement)
4. ✅ Section 3.3.3: Pattern Retirement Policy (Kaizen Enhancement)
5. ✅ Section 7: 10 additional peer-reviewed citations
