# Getting Started with Decy Development

**Decy**: Production-Grade C-to-Rust Transpiler with EXTREME TDD

---

## 📚 Understanding the Specification

Before starting development, familiarize yourself with these key documents:

### **Core Specifications** (Must Read)
1. **[decy-spec-v1.md](docs/specifications/decy-spec-v1.md)** - Complete technical specification
   - Architecture overview (multi-stage pipeline)
   - Quality requirements (80%+ coverage, ≥90% mutation score)
   - Development methodology (EXTREME TDD, Toyota Way)
   - 20-sprint roadmap with detailed tickets

2. **[decy-unsafe-minimization-strategy.md](docs/specifications/decy-unsafe-minimization-strategy.md)** - Unsafe code minimization
   - Multi-phase approach to reduce unsafe code from 100% → <5%
   - Pattern-based refactoring, ownership inference, safe wrappers
   - Property tests for safety verification

### **Quick Reference**
- **Quality Gates**: 80% coverage, ≤10 cyclomatic complexity, 0 SATD comments
- **Testing Strategy**: Unit + Property + Mutation + Integration + Book tests
- **Commit Strategy**: RED-GREEN-REFACTOR with atomic commits per ticket

---

## 🎯 Development Philosophy

Decy follows **EXTREME TDD** with these principles:

### **1. RED-GREEN-REFACTOR (Mandatory)**

Every ticket follows this workflow:

```bash
# RED Phase: Write failing tests first
git commit -m "[RED] DECY-001: Add failing tests for clang-sys integration"

# GREEN Phase: Minimal implementation to pass tests
git commit -m "[GREEN] DECY-001: Implement clang-sys parser"

# REFACTOR Phase: Clean up while maintaining green
git commit -m "[REFACTOR] DECY-001: Refactor parser to meet quality gates"
```

### **2. Zero Tolerance Policies**

- **SATD Comments**: NO `TODO`, `FIXME`, `HACK`, `XXX` allowed (blocked by pre-commit)
- **Coverage**: ≥80% at ALL times (enforced by pre-commit hook)
- **Linting**: 100% clippy pass, 0 warnings (blocked by pre-commit)
- **Unsafe Code**: ≤5 unsafe blocks per 1000 LOC in generated code

### **3. Quality Score**

Target: **A+ (98/100)** calculated as:
```
Score = 0.25 * complexity_score +
        0.25 * coverage_score +
        0.20 * mutation_score +
        0.15 * property_test_score +
        0.15 * linting_score
```

---

## 🚀 Setting Up Your Development Environment

### **Prerequisites**

```bash
# 1. Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Install LLVM/Clang (required for clang-sys)
# macOS:
brew install llvm

# Ubuntu/Debian:
sudo apt-get install llvm-dev libclang-dev clang

# 3. Install development tools
cargo install cargo-llvm-cov    # Fast coverage
cargo install cargo-mutants     # Mutation testing
cargo install mdbook            # Book generation
cargo install cargo-watch       # Auto-rebuild

# 4. Install quality tools
rustup component add clippy rustfmt
```

### **Clone and Build**

```bash
# Clone the repository
git clone https://github.com/your-org/decy.git
cd decy

# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Run quality gates
./scripts/quality-gates.sh
```

---

## 📖 Understanding the Codebase

### **Crate Structure**

```
decy/
├── crates/
│   ├── decy-core/          # Main transpilation pipeline
│   ├── decy-parser/        # C AST parsing (clang-sys)
│   ├── decy-hir/           # High-level Intermediate Representation
│   ├── decy-analyzer/      # Static analysis & type inference
│   ├── decy-ownership/     # Ownership & lifetime inference (CRITICAL)
│   ├── decy-verify/        # Safety property verification
│   ├── decy-codegen/       # Rust code generation
│   ├── decy-book/          # Book-based verification
│   ├── decy-agent/         # Background daemon (MCP server)
│   ├── decy-mcp/           # MCP tools for Claude Code
│   ├── decy-repo/          # GitHub repository transpilation
│   └── decy/               # CLI binary
```

### **Data Flow**

```
C Source
  ↓ decy-parser (clang-sys)
C AST
  ↓ decy-hir
HIR (High-level IR)
  ↓ decy-analyzer + decy-ownership
HIR + Ownership Info
  ↓ decy-verify
Verified HIR
  ↓ decy-codegen
Rust Code
  ↓ decy-book
Verified Rust Project
```

---

## 🎫 Working on Tickets (EXTREME TDD)

### **Step 1: Pick a Ticket**

Check `roadmap.yaml` for current sprint tickets:

```yaml
# Example: DECY-001 from Sprint 1
- id: DECY-001
  title: "Setup clang-sys integration"
  priority: critical
  requirements:
    - "clang-sys dependency added"
    - "Translation unit parsing works"
    - "AST traversal implemented"
  tests:
    - "test_clang_parses_simple_c"
    - "test_clang_handles_includes"
    - "proptest_clang_parsing_deterministic"
  acceptance:
    - "Can parse basic C files"
    - "Tests at 80%+ coverage"
    - "Linting passes"
```

### **Step 2: RED Phase - Write Failing Tests**

```rust
// crates/decy-parser/tests/parser_tests.rs

use decy_parser::CParser;

#[test]
fn test_clang_parses_simple_c() {
    let parser = CParser::new().unwrap();
    let source = "int main() { return 0; }";

    let ast = parser.parse(source).unwrap();

    // This will FAIL initially (RED phase)
    assert!(ast.functions().len() == 1);
    assert_eq!(ast.functions()[0].name(), "main");
}

#[test]
fn test_clang_handles_includes() {
    let parser = CParser::new().unwrap();
    let source = r#"
        #include <stdio.h>
        int main() { printf("Hello\n"); return 0; }
    "#;

    let ast = parser.parse(source).unwrap();

    // This will FAIL initially (RED phase)
    assert!(ast.includes().contains(&"stdio.h"));
}

// Property test (100+ required across project)
use proptest::prelude::*;

proptest! {
    #[test]
    fn proptest_clang_parsing_deterministic(
        c_code in valid_c_code_generator()
    ) {
        let parser = CParser::new().unwrap();

        let ast1 = parser.parse(&c_code).unwrap();
        let ast2 = parser.parse(&c_code).unwrap();

        // Property: Parsing is deterministic
        prop_assert_eq!(ast1, ast2);
    }
}
```

**Commit RED phase**:
```bash
git add -A
git commit -m "[RED] DECY-001: Add failing tests for clang-sys integration

- Added test_clang_parses_simple_c
- Added test_clang_handles_includes
- Added proptest_clang_parsing_deterministic
- All tests currently FAIL (expected for RED phase)
- Coverage: 0% (no implementation yet)"
```

### **Step 3: GREEN Phase - Minimal Implementation**

```rust
// crates/decy-parser/src/lib.rs

use clang::{Clang, Index};

pub struct CParser {
    clang: Clang,
    index: Index<'static>,
}

impl CParser {
    pub fn new() -> Result<Self, Error> {
        let clang = Clang::new()?;
        let index = Index::new(&clang, false, false);
        Ok(Self { clang, index })
    }

    pub fn parse(&self, source: &str) -> Result<Ast, Error> {
        // Minimal implementation to pass tests
        let tu = self.index.parser("temp.c")
            .arguments(&["-std=c11"])
            .parse(source)?;

        let ast = Ast::from_translation_unit(&tu)?;
        Ok(ast)
    }
}
```

**Run tests** (should pass now):
```bash
cargo test -p decy-parser
# ✅ test_clang_parses_simple_c ... ok
# ✅ test_clang_handles_includes ... ok
# ✅ proptest_clang_parsing_deterministic ... ok
```

**Commit GREEN phase**:
```bash
git add -A
git commit -m "[GREEN] DECY-001: Implement minimal clang-sys parser

- Implemented CParser with clang-sys
- All 3 tests now PASS
- Coverage: 65% (needs improvement in REFACTOR)
- Linting: 2 clippy warnings (will fix in REFACTOR)"
```

### **Step 4: REFACTOR Phase - Meet Quality Gates**

```rust
// Refactor to meet quality standards
impl CParser {
    /// Create a new C parser using clang-sys
    ///
    /// # Examples
    /// ```
    /// use decy_parser::CParser;
    ///
    /// let parser = CParser::new().unwrap();
    /// ```
    pub fn new() -> Result<Self, Error> {
        let clang = Clang::new()
            .map_err(|e| Error::ClangInitFailed(e.to_string()))?;
        let index = Index::new(&clang, false, false);
        Ok(Self { clang, index })
    }

    // ... more improvements
}
```

**Run quality gates**:
```bash
./scripts/quality-gates.sh
# ✅ Coverage: 82% (target: ≥80%)
# ✅ Clippy: 0 warnings
# ✅ Formatting: passed
# ✅ SATD: 0 comments
# ✅ Complexity: max 8 (target: ≤10)
# ✅ All tests pass
```

**Commit REFACTOR phase**:
```bash
git add -A
git commit -m "[REFACTOR] DECY-001: Refactor parser to meet quality gates

- Added documentation with doctests
- Improved error handling
- Increased coverage: 65% → 82%
- Fixed clippy warnings: 2 → 0
- Complexity: max 8 (within ≤10 target)
- Quality grade: A (95/100)"
```

### **Step 5: Final Atomic Commit**

Squash the 3 commits into one:
```bash
git rebase -i HEAD~3
# Mark first as "pick", others as "squash"

# Edit commit message:
git commit --amend -m "DECY-001: Setup clang-sys integration

- Implemented CParser with clang-sys
- Added AST parsing for C code
- Added translation unit support
- Added 3 tests (unit + property)
- Coverage: 82% (target: ≥80%) ✅
- Mutation score: 88% (target: ≥90%) ⚠️
- Quality grade: A (95/100) ✅
- Linting: 0 warnings ✅

Tests:
- test_clang_parses_simple_c
- test_clang_handles_includes
- proptest_clang_parsing_deterministic

Closes #1"
```

---

## 🧪 Testing Requirements

Every crate needs **4 types of tests**:

### **1. Unit Tests** (in `src/` or `tests/`)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_happy_path() {
        // Arrange, Act, Assert
    }

    #[test]
    fn test_feature_error_case() {
        // Test error handling
    }
}
```

### **2. Property Tests** (100+ across project)
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn property_never_panics(input in any::<String>()) {
        let _ = function_under_test(&input);
    }
}
```

### **3. Mutation Tests** (≥90% kill rate)
```bash
cargo mutants -p decy-parser
# Target: ≥90% of mutants killed
```

### **4. Integration Tests** (in `tests/`)
```rust
// tests/integration_test.rs

#[test]
fn test_end_to_end_transpilation() {
    let c_code = "int add(int a, int b) { return a + b; }";
    let rust_code = transpile(c_code).unwrap();

    assert!(compile_rust(&rust_code).is_ok());
    assert!(clippy_passes(&rust_code));
}
```

---

## 📊 Quality Gates (Enforced by Pre-commit)

Before every commit, these checks run automatically:

```bash
# 1. Coverage check
cargo llvm-cov --quiet | grep "TOTAL"
# Must be ≥80%

# 2. Linting
cargo clippy -- -D warnings
# Must have 0 warnings

# 3. Formatting
cargo fmt -- --check
# Must be formatted

# 4. SATD check
git diff --cached | grep -E "TODO|FIXME|HACK"
# Must have 0 matches

# 5. Tests
cargo test --all-features
# Must pass 100%

# 6. Complexity
cargo run --bin quality-check
# Max cyclomatic: ≤10, cognitive: ≤15
```

If any check fails, **commit is blocked**.

---

## 🌐 GitHub Repository Transpilation

Once basic transpilation works, you can transpile entire repos:

```bash
# Transpile Python's CPython to Rust
decy transpile-repo https://github.com/python/cpython \
    --output ./cpython-rust \
    --verify \
    --generate-book

# Output:
# - 487 C files → 481 Rust modules
# - Success rate: 98.8%
# - Unsafe blocks: 23 (<5 per 1000 LOC)
# - Full Cargo project + verification book
```

---

## 📈 Sprint Workflow

### **Current Sprint: Sprint 1 (Week 1-2)**

**Goal**: "Clang-based C AST parsing with 80% coverage"

**Tickets**:
1. **DECY-001**: Setup clang-sys integration ← START HERE
2. **DECY-002**: C type system extraction
3. **DECY-003**: Macro expansion handling

**Sprint Completion Checklist**:
- [ ] All tickets completed (RED-GREEN-REFACTOR)
- [ ] Sprint 1 coverage ≥80%
- [ ] Sprint 1 mutation score ≥90%
- [ ] All quality gates pass
- [ ] Documentation updated
- [ ] CHANGELOG.md updated

---

## 🛠️ Useful Commands

### **Development**
```bash
# Watch mode (auto-rebuild on changes)
cargo watch -x build -x test

# Run specific test
cargo test test_clang_parses_simple_c

# Run tests for specific crate
cargo test -p decy-parser

# Run with coverage
cargo llvm-cov --html
open target/llvm-cov/html/index.html
```

### **Quality Checks**
```bash
# Run all quality gates
./scripts/quality-gates.sh

# Run mutation tests
cargo mutants -p decy-parser

# Run property tests (longer)
cargo test --features proptest-tests -- --ignored

# Check complexity
cargo run --bin quality-check
```

### **Book Generation**
```bash
cd book
mdbook build
mdbook test
mdbook serve  # View at http://localhost:3000
```

---

## 🎓 Learning Resources

### **EXTREME TDD**
- Read: [Test-Driven Development by Kent Beck](https://www.amazon.com/Test-Driven-Development-Kent-Beck/dp/0321146530)
- Practice: Write tests BEFORE implementation (always)

### **Property-Based Testing**
- Read: [PropEr Testing](https://propertesting.com/)
- Tool: [proptest crate](https://docs.rs/proptest/)

### **Mutation Testing**
- Read: [Mutation Testing Overview](https://en.wikipedia.org/wiki/Mutation_testing)
- Tool: [cargo-mutants](https://mutants.rs/)

### **Toyota Way**
- **Jidoka** (自働化): Build quality in (pre-commit hooks)
- **Hansei** (反省): Reflect on bugs (Five Whys analysis)
- **Kaizen** (改善): Continuous improvement (weekly reviews)
- **Genchi Genbutsu** (現地現物): Go and see (dogfooding on real C projects)

---

## 🚨 Common Issues

### **Issue: Coverage below 80%**
```bash
# Find uncovered lines
cargo llvm-cov --html
# Add tests for uncovered code paths
```

### **Issue: Clippy warnings**
```bash
# See detailed warnings
cargo clippy --all-features -- -D warnings -W clippy::all

# Auto-fix some issues
cargo clippy --fix
```

### **Issue: Property tests failing**
```bash
# Get minimal failing case
cargo test proptest_name -- --nocapture

# Add to regression suite
# proptest generates .proptest-regressions/ files automatically
```

### **Issue: Mutation tests surviving**
```bash
# See which mutants survived
cargo mutants -p decy-parser --list

# Add tests to kill surviving mutants
```

---

## 🎯 Success Criteria

You're on track if:

- ✅ Every commit follows RED-GREEN-REFACTOR
- ✅ Coverage stays ≥80% at all times
- ✅ Clippy has 0 warnings
- ✅ No SATD comments (TODO/FIXME/HACK)
- ✅ Mutation score trending toward ≥90%
- ✅ Quality grade: A+ (98+) or A (95+)
- ✅ All tests pass (unit + property + integration)
- ✅ Generated Rust compiles and lints clean

---

## 📞 Getting Help

- **Specification Questions**: Read `docs/specifications/decy-spec-v1.md`
- **Quality Issues**: Check `docs/quality/standards.md`
- **Roadmap**: See `roadmap.yaml` for all tickets
- **Architecture**: See `docs/architecture/README.md`

---

## 🚀 Ready to Start?

1. **Read** the specification (15 min)
2. **Setup** development environment (10 min)
3. **Start** DECY-001 with RED phase (30 min)
4. **Iterate** with GREEN and REFACTOR (1-2 hours)
5. **Celebrate** your first ticket completion! 🎉

**Next**: See `OPTION A` setup for complete project initialization, then start `OPTION B` (Sprint 1, DECY-001).

---

**Good luck, and remember: Tests first, quality always!** ✨
