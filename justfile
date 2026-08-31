# Task runner for the rust-agents-template repo.
# `just` (no arguments) lists all recipes.

default:
    @just --list

# One-time setup per clone: activate git hooks + install missing check tools.
setup:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "$(git rev-parse --show-toplevel)"
    git config core.hooksPath githooks
    echo "hooksPath -> githooks"
    for tool in cargo-machete cargo-audit cargo-outdated cargo-deny; do
        if command -v "$tool" >/dev/null 2>&1; then
            echo "ok:      $tool"
        else
            echo "install: $tool"
            cargo install "$tool" --locked
        fi
    done
    echo "setup complete"

# Auto-fix formatting across the workspace.
fmt:
    cargo fmt --all

# Run tests (cargo-nextest when installed, otherwise cargo test).
test:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "$(git rev-parse --show-toplevel)"
    if command -v cargo-nextest >/dev/null 2>&1; then
        cargo nextest run
    else
        cargo test
    fi

# Run the full check chain (identical to hooks + CI: fmt/machete/docs/clippy + audit/deny/outdated/test).
check:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "$(git rev-parse --show-toplevel)"
    githooks/pre-commit
    githooks/pre-push
