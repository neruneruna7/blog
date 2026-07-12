# Typst Blog Engine

Current milestone:
- Rust content engine generates site/article JSON.
- Rust content engine generates Typst-derived body.html fragments.
- Astro reads generated/ and builds static pages.
- React is not introduced yet.
- Typst HTML export is experimental and intentionally accepted for this prototype.

Commands:
- cargo make build-content
- cd frontend && npm install
- cargo make build-site
- cargo make build
- cargo make test
- cargo make test-content
- cargo make check
- cargo make clean

Typst CLI:
- `cargo make build-content` uses `typst compile --features html` to generate `generated/articles/<slug>/body.html`.
- `body.html` is an intermediate HTML fragment. It is not a final page.

Article source layout:
- Article configuration is discovered under `typst-src/articles/<directory-name>/ArticleConfig.toml`.
- The article directory name is a source-layout name independent of the public slug in `ArticleConfig.toml`.
- Public URLs and generated article output directories are derived from the configured slug.
- New article creation uses the generated slug as the default directory name.

Astro:
- `cargo make build-site` builds `frontend/dist/` from the existing files under `generated/`.
- `cargo make build` runs content generation first, then builds the Astro static site.
- `cargo make test` runs Rust content tests and frontend build-level assertions.
- Run `cd frontend && npm install` before the first Astro build. Dependency installation is intentionally not hidden inside cargo-make.

Optional real Typst integration test:
- BLOG_ENGINE_RUN_TYPST_INTEGRATION=1 cargo test -p blog-engine

- UI primitiveは原則 shadcn/ui を使う.
- スタイリングは原則 Tailwind utility class を使う.
- サイト固有componentは src/components/site に置く.
- shadcn由来componentは src/components/ui に置く.
- 記事本文HTMLはRust生成物だけを埋め込む.
