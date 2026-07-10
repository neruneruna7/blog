# Task Completion

- Run `cargo fmt --all` or `cargo make fmt`.
- Run `cargo clippy --workspace --all-targets -- -D warnings` or `cargo make clippy`.
- Run `cargo test -p blog-engine` or `cargo make test-content`.
- Run `cargo make build-content` when metadata output behavior changes.
- Verify `generated/` remains ignored and no final HTML frontend artifacts are introduced.