# Decy Makefile - Reproducible Development Environment
# All commands needed to setup, build, test, and develop Decy

.PHONY: help install install-rust install-llvm install-tools check-llvm \
        build test test-fast test-all test-unit test-integration test-doc \
        test-examples coverage mutation lint fmt check clean quality-gates \
        verify-install pre-commit-setup

# Default target
.DEFAULT_GOAL := help

##@ Help

help: ## Display this help message
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Installation

install: install-rust install-llvm install-tools check-llvm verify-install ## Complete installation (Rust + LLVM + Tools)
	@echo ""
	@echo "✅ Installation complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Reload shell or run: source ~/.cargo/env"
	@echo "  2. Run: make build"
	@echo "  3. Run: make test"
	@echo ""

install-rust: ## Install Rust toolchain (if not present)
	@echo "🦀 Checking Rust installation..."
	@if command -v rustc >/dev/null 2>&1; then \
		echo "✅ Rust already installed: $$(rustc --version)"; \
	else \
		echo "📦 Installing Rust..."; \
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; \
		echo "✅ Rust installed"; \
	fi
	@echo "📦 Updating Rust to latest stable..."
	@. ~/.cargo/env && rustup update stable
	@echo "📦 Setting default toolchain to stable..."
	@. ~/.cargo/env && rustup default stable
	@echo "📦 Adding required components..."
	@. ~/.cargo/env && rustup component add rustfmt clippy llvm-tools-preview
	@echo "✅ Rust toolchain ready: $$(. ~/.cargo/env && rustc --version)"

install-llvm: ## Install LLVM/Clang development libraries
	@echo "🔧 Installing LLVM/Clang development libraries..."
	@if [ -f /etc/debian_version ]; then \
		echo "📦 Detected Debian/Ubuntu"; \
		sudo apt-get update; \
		sudo apt-get install -y \
			llvm-14-dev \
			libclang-14-dev \
			clang-14 \
			llvm-14 \
			libclang1-14 \
			build-essential \
			pkg-config; \
		echo "🔗 Setting up LLVM environment variables..."; \
		echo "export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14" >> ~/.bashrc; \
		echo "export LIBCLANG_PATH=/usr/lib/llvm-14/lib" >> ~/.bashrc; \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	elif [ -f /etc/redhat-release ]; then \
		echo "📦 Detected RHEL/CentOS/Fedora"; \
		sudo yum install -y llvm-devel clang-devel clang || \
		sudo dnf install -y llvm-devel clang-devel clang; \
	elif [ "$$(uname)" = "Darwin" ]; then \
		echo "📦 Detected macOS"; \
		if ! command -v brew >/dev/null 2>&1; then \
			echo "❌ Homebrew not found. Please install from https://brew.sh"; \
			exit 1; \
		fi; \
		brew install llvm; \
		echo "🔗 Setting up LLVM environment variables for macOS..."; \
		echo 'export PATH="/usr/local/opt/llvm/bin:$$PATH"' >> ~/.zshrc; \
		echo 'export LDFLAGS="-L/usr/local/opt/llvm/lib"' >> ~/.zshrc; \
		echo 'export CPPFLAGS="-I/usr/local/opt/llvm/include"' >> ~/.zshrc; \
	else \
		echo "❌ Unsupported operating system"; \
		echo "Please install LLVM/Clang manually:"; \
		echo "  - LLVM development libraries"; \
		echo "  - Clang development libraries"; \
		echo "  - Set LLVM_CONFIG_PATH and LIBCLANG_PATH environment variables"; \
		exit 1; \
	fi
	@echo "✅ LLVM/Clang libraries installed"

install-tools: ## Install Rust development tools
	@echo "🔧 Installing Rust development tools..."
	@. ~/.cargo/env && cargo install cargo-llvm-cov || echo "cargo-llvm-cov already installed"
	@. ~/.cargo/env && cargo install cargo-mutants || echo "cargo-mutants already installed"
	@. ~/.cargo/env && cargo install cargo-watch || echo "cargo-watch already installed"
	@. ~/.cargo/env && cargo install cargo-edit || echo "cargo-edit already installed"
	@echo "✅ Development tools installed"

check-llvm: ## Verify LLVM/Clang installation
	@echo "🔍 Checking LLVM/Clang installation..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi
	@if command -v llvm-config >/dev/null 2>&1; then \
		echo "✅ llvm-config found: $$(llvm-config --version)"; \
	elif command -v llvm-config-14 >/dev/null 2>&1; then \
		echo "✅ llvm-config-14 found: $$(llvm-config-14 --version)"; \
	else \
		echo "❌ llvm-config not found"; \
		exit 1; \
	fi
	@if command -v clang >/dev/null 2>&1; then \
		echo "✅ clang found: $$(clang --version | head -1)"; \
	elif command -v clang-14 >/dev/null 2>&1; then \
		echo "✅ clang-14 found: $$(clang-14 --version | head -1)"; \
	else \
		echo "❌ clang not found"; \
		exit 1; \
	fi
	@echo "✅ LLVM/Clang verification passed"

verify-install: ## Verify complete installation
	@echo "🔍 Verifying installation..."
	@echo ""
	@echo "Rust toolchain:"
	@. ~/.cargo/env && rustc --version
	@. ~/.cargo/env && cargo --version
	@. ~/.cargo/env && rustfmt --version
	@. ~/.cargo/env && cargo clippy --version
	@echo ""
	@echo "LLVM/Clang:"
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi; \
	if command -v llvm-config-14 >/dev/null 2>&1; then \
		llvm-config-14 --version; \
	elif command -v llvm-config >/dev/null 2>&1; then \
		llvm-config --version; \
	fi
	@if command -v clang-14 >/dev/null 2>&1; then \
		clang-14 --version | head -1; \
	elif command -v clang >/dev/null 2>&1; then \
		clang --version | head -1; \
	fi
	@echo ""
	@echo "Development tools:"
	@. ~/.cargo/env && cargo llvm-cov --version || echo "cargo-llvm-cov not installed (optional)"
	@. ~/.cargo/env && cargo mutants --version || echo "cargo-mutants not installed (optional)"
	@echo ""
	@echo "✅ Verification complete!"

pre-commit-setup: ## Setup git pre-commit hooks
	@echo "🔧 Setting up pre-commit hooks..."
	@chmod +x scripts/quality-gates.sh
	@chmod +x .git/hooks/pre-commit
	@echo "✅ Pre-commit hooks enabled"

##@ Build

build: ## Build all crates in workspace
	@echo "🔨 Building workspace..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo build --workspace --all-features

build-release: ## Build optimized release binaries
	@echo "🔨 Building release..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo build --workspace --all-features --release

##@ Testing

test: ## Run all tests (unit + integration + doc)
	@echo "🧪 Running all tests..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo test --workspace --all-features

test-fast: ## Run only unit tests (fast)
	@echo "🧪 Running unit tests..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo test --workspace --lib

test-unit: test-fast ## Alias for test-fast

test-integration: ## Run integration tests
	@echo "🧪 Running integration tests..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo test --workspace --test '*'

test-doc: ## Run documentation tests
	@echo "🧪 Running doc tests..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo test --workspace --doc

test-examples: ## Run example tests
	@echo "🧪 Running example tests..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo test --workspace --examples

test-all: test test-doc test-examples ## Run complete test suite

##@ Quality

coverage: ## Generate comprehensive test coverage report
	@echo "📊 Running comprehensive test coverage analysis..."
	@echo "🔍 Checking for cargo-llvm-cov..."
	@which cargo-llvm-cov > /dev/null 2>&1 || (echo "📦 Installing cargo-llvm-cov..." && cargo install cargo-llvm-cov --locked)
	@echo "🧹 Cleaning old coverage data..."
	@cargo llvm-cov clean --workspace
	@mkdir -p target/coverage
	@echo "🧪 Phase 1: Running tests with instrumentation (no report)..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
		cargo llvm-cov --no-report --workspace --all-features; \
	else \
		cargo llvm-cov --no-report --workspace --all-features; \
	fi
	@echo "📊 Phase 2: Generating coverage reports..."
	@cargo llvm-cov report --html --output-dir target/coverage/html
	@cargo llvm-cov report --lcov --output-path target/coverage/lcov.info
	@echo ""
	@echo "📊 Coverage Summary:"
	@echo "=================="
	@cargo llvm-cov report --summary-only
	@echo ""
	@echo "💡 COVERAGE INSIGHTS:"
	@echo "- HTML report: target/coverage/html/index.html"
	@echo "- LCOV file: target/coverage/lcov.info"
	@echo ""

coverage-summary: ## Show coverage summary only (fast)
	@echo "📊 Coverage summary:"
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo llvm-cov --workspace --all-features --summary-only

mutation: ## Run mutation tests (requires cargo-mutants, slow)
	@echo "🧬 Running mutation tests (this may take a while)..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo mutants --workspace --output mutations.json
	@echo "✅ Mutation test results: mutations.json"

lint: ## Run clippy lints (zero warnings policy)
	@echo "🔍 Running clippy..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt: ## Format code with rustfmt
	@echo "📝 Formatting code..."
	@cargo fmt --all

fmt-check: ## Check code formatting
	@echo "📝 Checking formatting..."
	@cargo fmt --all -- --check

check: ## Run basic checks (build + lint + fmt + test)
	@echo "✅ Running basic checks..."
	@make fmt-check
	@make lint
	@make build
	@make test-fast

quality-gates: ## Run all quality gates (pre-commit checks)
	@echo "🔍 Running quality gates..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && ./scripts/quality-gates.sh

##@ Documentation

doc: ## Build documentation
	@echo "📚 Building documentation..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo doc --workspace --no-deps --document-private-items

doc-open: doc ## Build and open documentation in browser
	@echo "📚 Opening documentation..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo doc --workspace --no-deps --document-private-items --open

##@ Cleanup

clean: ## Remove build artifacts
	@echo "🧹 Cleaning..."
	@cargo clean
	@rm -rf target/
	@rm -rf mutations.json
	@rm -rf mutants.out/
	@rm -rf lcov.info
	@rm -rf cobertura.xml
	@echo "✅ Clean complete"

##@ Development

watch: ## Watch for changes and run tests
	@echo "👀 Watching for changes..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo watch -x "test --workspace"

watch-check: ## Watch for changes and run checks
	@echo "👀 Watching for changes (check mode)..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo watch -x "check --workspace"

run: ## Run the decy CLI (development mode)
	@echo "🚀 Running decy..."
	@if [ -f /etc/debian_version ]; then \
		export LLVM_CONFIG_PATH=/usr/bin/llvm-config-14; \
		export LIBCLANG_PATH=/usr/lib/llvm-14/lib; \
	fi && cargo run -p decy

##@ Sprint Management

sprint-status: ## Show current sprint status
	@echo "📊 Sprint Status:"
	@echo ""
	@echo "Current Sprint: Sprint 1 - Foundation & C Parser"
	@echo "Version: 0.1.0"
	@echo ""
	@echo "Tickets:"
	@echo "  [ ] DECY-001: Setup clang-sys integration (RED phase)"
	@echo "  [ ] DECY-002: Define HIR structure"
	@echo "  [ ] DECY-003: Implement basic code generator"
	@echo ""
	@echo "See roadmap.yaml for full details"

quality-metrics: coverage-summary ## Show quality metrics
	@echo ""
	@echo "📊 Quality Metrics:"
	@echo ""
	@echo "Targets:"
	@echo "  • Coverage: ≥80% (target: 85%)"
	@echo "  • Mutation kill rate: ≥90%"
	@echo "  • Clippy warnings: 0"
	@echo "  • SATD comments: 0"
	@echo "  • Unsafe blocks: <5 per 1000 LOC"
	@echo ""

##@ CI/CD

ci-local: ## Run CI checks locally
	@echo "🔄 Running CI checks locally..."
	@make fmt-check
	@make lint
	@make build
	@make test-all
	@make coverage-summary
	@echo "✅ All CI checks passed!"

##@ Information

info: ## Show project information
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "  Decy: C-to-Rust Transpiler"
	@echo "  Version: 0.1.0"
	@echo "  EXTREME TDD + Toyota Way + PMAT Qualified"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "📚 Documentation:"
	@echo "  • Getting Started: GETTING_STARTED.md"
	@echo "  • Specification: docs/specifications/decy-spec-v1.md"
	@echo "  • Roadmap: roadmap.yaml"
	@echo "  • Quality Config: decy-quality.toml"
	@echo ""
	@echo "🔧 Quick Commands:"
	@echo "  • make install      - Complete installation"
	@echo "  • make build        - Build workspace"
	@echo "  • make test         - Run all tests"
	@echo "  • make quality-gates - Run quality checks"
	@echo "  • make help         - Show all commands"
	@echo ""

version: ## Show version information
	@echo "Decy version: 0.1.0"
	@echo "Sprint: 1 (Foundation & C Parser)"
	@. ~/.cargo/env && rustc --version 2>/dev/null || echo "Rust: not installed"
	@. ~/.cargo/env && cargo --version 2>/dev/null || echo "Cargo: not installed"

##@ PMAT Enforcement

sync-roadmap: ## Sync roadmap.yaml with GitHub Issues
	@echo "📋 Synchronizing roadmap with GitHub Issues..."
	@./scripts/sync-roadmap.sh

check-roadmap: ## Verify roadmap state integrity
	@echo "🔍 Checking roadmap state..."
	@echo "Current Sprint: Sprint 1"
	@echo "Active Tickets:"
	@grep -A 3 "status: in_progress" roadmap.yaml || echo "  No active tickets"
	@echo ""
	@echo "Pending Tickets:"
	@grep -A 3 "status: not_started" roadmap.yaml | head -20 || echo "  No pending tickets"

roadmap-status: ## Show roadmap status
	@echo "📊 Roadmap Status:"
	@echo ""
	@echo "Sprint 1 - Foundation & C Parser"
	@echo "  DECY-001: in_progress (RED phase)"
	@echo "  DECY-002: not_started"
	@echo "  DECY-003: not_started"
	@echo ""
	@echo "Run 'make sync-roadmap' to create GitHub issues"

# ============================================================================
# BOOK TARGETS (TDD-Enforced Documentation)
# ============================================================================

.PHONY: book book-test book-build book-serve book-clean

book: book-test book-build ## Build and test the mdBook (TDD-enforced)

book-test: ## Test all code examples in the book (CI blocking)
	@echo "📖 Testing book code examples..."
	@./scripts/test-book.sh

book-build: ## Build the mdBook HTML
	@echo "📖 Building book..."
	@cd book && mdbook build

book-serve: ## Serve the book locally (http://localhost:3000)
	@echo "📖 Serving book at http://localhost:3000"
	@cd book && mdbook serve --open

book-clean: ## Clean book build artifacts
	@rm -rf book/book/
	@echo "✓ Book artifacts cleaned"

