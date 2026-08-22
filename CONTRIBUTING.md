# Contributing to `tui-breadcrumbs`

Thank you for your interest in contributing to `tui-breadcrumbs`! Whether you are reporting a bug, proposing a new feature, improving documentation, or submitting code changes, your contributions are warmly welcome.

---

## 🛠️ Development Setup

### Prerequisites
- **Rust Toolchain**: Stable Rust 1.85+ (Edition 2024 supported).
- **Cargo Components** (recommended):
  ```bash
  rustup component add clippy rustfmt
  ```

### Getting Started
1. **Fork and Clone** the repository:
   ```bash
   git clone https://github.com/shadowmkj/tui-breadcrumbs.git
   cd tui-breadcrumbs
   ```
2. **Build the project**:
   ```bash
   cargo build --all-targets
   ```
3. **Run the interactive demo**:
   ```bash
   cargo run --example demo
   ```

---

## 🧪 Testing & Validation

Before submitting a pull request, ensure all tests, lints, and documentation checks pass cleanly.

### Running Test Suites
```bash
# Run unit tests
cargo test --lib

# Run doctests
cargo test --doc

# Run property-based invariant tests (QuickCheck)
cargo test --test property_test

# Run snapshot regression tests (Insta)
cargo test --test snapshots

# Run all tests across all targets and features
cargo test --all-targets --all-features
```

### Code Formatting & Linting
```bash
# Check formatting
cargo fmt --all -- --check

# Run Clippy with strict warnings
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation Verification
```bash
# Ensure docs compile with zero broken intra-doc links or warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
```

---

## 📐 Code Style & Architecture Guidelines

When contributing code to `tui-breadcrumbs`, please adhere to the following principles:

1. **Rust API Guidelines**:
   - Follow standard Rust API guidelines (<https://rust-lang.github.io/api-guidelines/>).
   - Implement standard traits where appropriate (`Debug`, `Clone`, `PartialEq`, `Eq`, `Default`).
   - Use builder methods with `#[must_use]` where suitable.

2. **Accurate Width Calculations**:
   - Use `unicode-width` (`UnicodeWidthStr`) for measuring terminal columns rather than byte length or character count.
   - Respect multi-byte UTF-8 character boundaries (`char_indices`) to prevent slicing mid-character.

3. **Separation of Concerns**:
   - Keep layout and truncation math (`src/truncate.rs`) decoupled from terminal rendering (`src/widget.rs`).
   - Maintain dual support for stateless [`ratatui::widgets::Widget`] and stateful [`ratatui::widgets::StatefulWidget`].

4. **Literate & Thorough Documentation**:
   - Write clear doc comments explaining *why* decisions were made, not just *what* the code does.
   - Provide realistic examples in doc comments.

---

## 🌿 Git & Pull Request Workflow

1. **Create a Feature Branch**:
   ```bash
   git checkout -b feat/my-new-feature
   ```
2. **Commit Conventions**:
   - Use semantic commit messages:
     - `feat:` for new features or capabilities
     - `fix:` for bug fixes
     - `docs:` for documentation updates
     - `test:` for adding or updating tests
     - `refactor:` for code refactoring without behavioral changes
     - `perf:` for performance improvements
     - `chore:` for maintenance tasks, tooling, or dependency updates
3. **Open a Pull Request**:
   - Provide a concise summary of your changes.
   - Reference any related issues or discussions.
   - Verify that all CI workflow checks pass on your PR branch.

---

## 📄 License

By submitting a contribution, you agree that your contributions will be dual-licensed under the terms of both the **MIT License** and the **Apache License, Version 2.0**.
