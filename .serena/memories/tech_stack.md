# Tech Stack

- Rust workspace.
- Main crate: `crates/blog-engine`.
- Task runner: cargo-make via `Makefile.toml`; do not add `justfile` or root `Makefile` for current workflow.
- Metadata parsing: TOML input into typed Rust structs; JSON output via serde.
- Date validation: RFC3339 datetimes via chrono.
- Directory traversal: walkdir.
- Domain errors: thiserror with anyhow only at the thin CLI boundary.