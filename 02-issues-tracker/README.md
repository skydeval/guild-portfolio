# Issue Tracker

Phase 2 portfolio project for the [Navigators Guild](https://github.com/Navigators-Guild) apprentice path. A personal issue tracker CLI in Rust. Tracks tasks in a local JSON file in the current directory.

## Files

- `DESIGN.md` — design document (purpose, features, technology, interface, out of scope)
- `Cargo.toml` / `Cargo.lock` — Rust project manifest and resolved dependencies
- `src/main.rs` — the entire program, single file
- `PROCESS.md` — build process documentation, including review-driven changes

## Building and running

```
cargo build --release
./target/release/tracker --help
```

For development:

```
cargo run -- --help
cargo run -- create "Fix the login bug" --priority high --label bug
cargo run -- list
```

## Storage

Issues are stored in `tracker.json` in the current working directory. The file is wrapped in a small envelope:

```json
{
  "schema_version": 1,
  "next_id": 5,
  "issues": [ ... ]
}
```

The `next_id` field guarantees that issue ids are stable: deleting an issue does not free its number for reuse.

Concurrent invocations are protected by a sidecar `tracker.json.lock` file (advisory exclusive lock). Atomic writes use a `tracker.json.tmp` staging file followed by `rename(2)`.

## Scope

This is the curriculum-spec version: the eight features defined in `DESIGN.md`, built in layers per chapter 02 of the apprentice methodology guide. Round 1 adversarial review feedback has been addressed in subsequent commits.