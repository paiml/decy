# Gemini Project Guide: Decy

This document provides a guide for Gemini to effectively understand, build, test, and contribute to the Decy project.

## 1. Project Overview

Decy is a production-grade, quality-focused C-to-Rust transpiler. Its primary goal is to convert legacy C codebases into safe, idiomatic Rust, with a strong emphasis on minimizing the use of `unsafe` blocks.

The project follows a stringent development methodology called "EXTREME TDD," inspired by the Toyota Way and PMAT qualification. This methodology enforces exceptionally high quality standards.

## 2. Architecture

Decy is a Rust workspace composed of multiple, modular crates that form a transpilation pipeline. Each crate has a specific responsibility:

-   `decy-parser`: Parses C code into an Abstract Syntax Tree (AST) using `clang-sys`.
-   `decy-hir`: Defines the High-level Intermediate Representation (HIR), an AST that is more Rust-oriented.
-   `decy-analyzer`: Performs static analysis and type inference on the HIR.
-   `decy-ownership`: Infers Rust ownership semantics (e.g., `&T`, `&mut T`, `Box<T>`) from C pointers.
-   `decy-verify`: Verifies safety properties of the code during transformation.
-   `decy-codegen`: Generates the final Rust code from the processed HIR.
-   `decy-book`: Manages "book-based verification," where documentation is compiled and tested.
-   `decy-agent`: A background daemon for incremental transpilation.
-   `decy-mcp`: A server for integration with external AI agents.
-   `decy-repo`: Handles the logic for transpiling entire Git repositories.
-   `decy`: The main command-line interface (CLI) application.

## 3. Development Workflow & Commands

The project uses a `Makefile` and a GitHub Actions workflow (`.github/workflows/quality.yml`) to standardize development tasks. These are the primary commands to use.

### 3.1. Setup

The project has a one-command setup process that installs Rust, LLVM/Clang, and all necessary development tools.

-   **Full Installation:**
    ```bash
    make install
    ```

### 3.2. Building

-   **Standard Build (Debug):**
    ```bash
    make build
    ```
-   **Release Build (Optimized):**
    ```bash
    make build-release
    ```

### 3.3. Testing

The project has a comprehensive testing suite.

-   **Run All Tests (Unit, Integration, Doc):**
    ```bash
    make test
    ```
-   **Generate Code Coverage Report:**
    ```bash
    make coverage
    ```
    *(The project enforces a minimum of 80% coverage.)*

-   **Run Mutation Tests:**
    ```bash
    make mutation
    ```
    *(The project aims for a 90%+ mutation kill rate.)*

### 3.4. Quality Gates (Pre-Commit Checks)

A pre-commit hook runs all essential quality checks. These should always be run before committing code.

-   **Run All Quality Gates:**
    ```bash
    make quality-gates
    ```
    This command executes the following checks:
    1.  **Formatting:** `cargo fmt --all -- --check`
    2.  **Linting:** `cargo clippy --workspace --all-targets --all-features -- -D warnings` (zero-warning policy)
    3.  **Testing:** `cargo test --workspace`
    4.  **SATD Check:** No `TODO`, `FIXME`, etc. comments are allowed.
    5.  **Doc Check:** `cargo doc --workspace --no-deps`

### 3.5. Documentation

-   **Build Project Documentation:**
    ```bash
    make doc
    ```
-   **Build and Open in Browser:**
    ```bash
    make doc-open
    ```

## 4. Key Files for Context

-   **`README.md`**: High-level project overview, goals, and quick start.
-   **`Cargo.toml` (root)**: Defines the workspace members and shared dependencies.
-   **`Makefile`**: The source of truth for all common development commands.
-   **`.github/workflows/quality.yml`**: Defines the CI pipeline and quality checks.
-   **`roadmap.yaml`**: The 20-sprint development plan.
-   **`docs/specifications/`**: Contains the detailed technical specifications for the transpiler.
