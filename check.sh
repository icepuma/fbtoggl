#!/usr/bin/env bash
set -euo pipefail

cargo fmt
cargo clippy --all-targets --all-features
cargo nextest run --all-targets
