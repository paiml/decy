# Decy Book (TDD-Enforced Documentation)

This is the official **mdBook** for Decy, built with **TDD-enforced examples**.

## 🔒 TDD Enforcement

**ALL Rust code examples in this book are tested in CI!**

- Every ```rust code block is compiled
- All `#[test]` functions are run
- **Release is BLOCKED** if any example fails

This ensures:
1. ✅ Examples actually work
2. ✅ Documentation stays up-to-date
3. ✅ No broken code ships to users

## Building the Book

### Prerequisites

Install mdBook:
```bash
cargo install mdbook
```

### Local Development

```bash
# Test all code examples (same as CI)
make book-test

# Build the book
make book-build

# Serve locally with live-reload
make book-serve
```

The book will be available at http://localhost:3000

## Structure

```
book/
├── book.toml           # mdBook configuration
├── src/
│   ├── SUMMARY.md      # Table of contents
│   ├── introduction.md
│   ├── patterns/       # C-to-Rust pattern chapters
│   ├── advanced/       # Advanced topics
│   └── reference/      # CLI/config reference
└── book/               # Generated HTML (gitignored)
```

## Writing Chapters

### Code Examples

**Always include tests** with your examples:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add() {
    assert_eq!(add(2, 2), 4);
}
```

This will be **compiled and tested** in CI!

### Non-Runnable Code

If you need to show code that shouldn't be tested:

```rust,ignore
// This won't be tested
fn hypothetical_example() { ... }
```

Or for code that should compile but not run:

```rust,no_run
fn main() {
    // Compiles but doesn't run in CI
}
```

## CI Pipeline

### On Every PR

1. **Test Examples**: All Rust code blocks are compiled and tested
2. **Build Book**: HTML is generated
3. **Link Check**: Broken links are detected (future)

### On Main Branch

1. All PR checks pass
2. Book is deployed to GitHub Pages
3. Available at https://paiml.github.io/decy

## Release Blocking

**The book MUST build and all examples MUST pass before release.**

If `make book-test` fails:
- ❌ PR checks fail
- ❌ Cannot merge to main
- ❌ Cannot create release

This is enforced by:
- `.github/workflows/book.yml` - CI workflow
- `scripts/quality-gates.sh` - Pre-commit hook
- `scripts/test-book.sh` - Local/CI test script

## Adding New Chapters

1. Create `.md` file in `book/src/`
2. Add to `SUMMARY.md`
3. Include tested code examples
4. Run `make book-test` locally
5. Commit when tests pass

## Philosophy

> **"Documentation that doesn't compile is a lie waiting to happen."**

By testing every example, we ensure the book is always:
- ✅ Accurate
- ✅ Up-to-date
- ✅ Trustworthy

This is part of Decy's **EXTREME TDD** and **Toyota Way** (Jidoka - build quality in) principles.

## Troubleshooting

### mdbook test fails

```bash
cd book
mdbook test
```

Fix the failing code examples. They must compile and pass tests.

### mdbook not found

```bash
cargo install mdbook
```

### Examples need crate dependencies

Add them to `[dependencies]` blocks in code:

```rust
// Test dependencies can be added inline
#[test]
fn test_with_external_crate() {
    // If you need external crates, discuss with maintainers
}
```

## More Information

- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [USER_GUIDE.md](../docs/USER_GUIDE.md) - Comprehensive user guide
- [CLAUDE.md](../CLAUDE.md) - Development guide

---

**Remember**: Every code example is a promise to our users. Make it a tested promise!
