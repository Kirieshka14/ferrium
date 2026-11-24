#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
MANIFEST="$ROOT/Cargo.toml"

case "${1:-}" in
  editor)
    shift
    exec cargo run --manifest-path "$MANIFEST" -p ferrium_editor -- "$@"
    ;;
  game)
    shift
    exec cargo run --manifest-path "$MANIFEST" -p ferrium_engine -- "$@"
    ;;
  release)
    shift
    exec cargo run --manifest-path "$MANIFEST" --release -p ferrium_editor -- "$@"
    ;;
  test)
    shift
    exec cargo test --manifest-path "$MANIFEST" --workspace "$@"
    ;;
  headless)
    shift
    cargo run --manifest-path "$MANIFEST" -p ferrium_editor -- --headless-check
    exec cargo run --manifest-path "$MANIFEST" -p ferrium_engine -- --headless-ticks "${1:-240}"
    ;;
  *)
    exec cargo run --manifest-path "$MANIFEST" -p ferrium_editor -- "$@"
    ;;
esac
