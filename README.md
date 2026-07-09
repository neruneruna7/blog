# Typst Blog Engine

Current milestone:
- Rust metadata engine
- ArticleConfig.toml -> generated JSON
- main.typ -> generated article body HTML fragment
- No Astro/React yet
- No OGP yet

Commands:
- cargo make build-content
- cargo make test-content
- cargo make check
- cargo make clean

Typst CLI:
- `cargo make build-content` uses `typst compile --features html` to generate `generated/articles/<slug>/body.html`.
- `body.html` is an intermediate HTML fragment. It is not a final page.

Optional real Typst integration test:
- BLOG_ENGINE_RUN_TYPST_INTEGRATION=1 cargo test -p blog-engine
