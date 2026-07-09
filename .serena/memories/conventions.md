# Conventions

- Library-first Rust design; keep `main.rs` thin and put domain logic in `lib.rs` modules.
- Public Rust modules, structs, enums, and functions require Japanese doc comments using the exact three-section format requested in project instructions.
- Prefer typed domain structs over raw JSON maps.
- Generated JSON keys are camelCase; TOML input fields remain snake_case.
- `updated_at = ""` maps to JSON `updatedAt: null`.
- Article slugs must match `^[a-z0-9]+(?:-[a-z0-9]+)*$`.
- Do not implement Typst compilation, HTML generation, Astro, React, Askama, OGP image generation, `justfile`, or root `Makefile` in the current milestone.