# Typst Blog Engine

Current milestone:
- Rust generates site/article JSON, Typst-derived `body.html`, and OGP assets.
- Astro reads generated content and `generated/public-root` to build static pages.
- React is not introduced yet.
- Typst HTML export is experimental and intentionally accepted for this prototype.

Commands:
- cargo make build-blog
- cd frontend && npm install
- cargo make build-frontend
- cargo make build
- cargo make test
- cargo make test-blog
- cargo make check
- cargo make clean

Blog generation:
- `cargo make build-blog` runs `blog-compiler build` to generate `body.html`, article metadata, and OGP assets.
- `cargo make build-blog-release` generates artifacts only for articles with `publish = true` in `ArticleConfig.toml`.
- New articles default to `publish = false`; the regular `build-blog` task includes all articles for development.
- OGP images are written under `generated/public-root/ogp/articles`.
- Generated article metadata keeps site-root-relative paths; Astro resolves the deployment origin from `BASE_URL` when it writes OGP tags.
- `body.html` is an HTML fragment embedded by Astro. It is not a final page.

Article source layout:
- Article configuration is discovered under `typst-src/articles/<directory-name>/ArticleConfig.toml`.
- The article directory name is a source-layout name independent of the public slug in `ArticleConfig.toml`.
- Public URLs and generated article output directories are derived from the configured slug.
- New article creation uses the generated slug as the default directory name.

Astro:
- `cargo make build-frontend` generates blog artifacts, merges `frontend/public` into `generated/public-root`, and builds `frontend/dist/`.
- `cargo make build` runs blog generation first, then builds the Astro frontend.
- `cargo make dev-blog` builds the initial artifacts, then starts `watch-blog` and `run-frontend` concurrently. The two long-running jobs remain separate process trees under one cargo-make command.
- `cargo make test` runs blog compiler tests and frontend build-level assertions.
- Set `BASE_URL` in the repository-root `.env` for local Astro builds; CI reads it from the GitHub Actions variable `BASE_URL`.
- Run `cd frontend && npm install` before the first Astro build. Dependency installation is intentionally not hidden inside cargo-make.

Optional real Typst integration test:
- BLOG_ENGINE_RUN_TYPST_INTEGRATION=1 cargo test -p blog-engine

- UI primitiveは原則 shadcn/ui を使う.
- スタイリングは原則 Tailwind utility class を使う.
- サイト固有componentは src/components/site に置く.
- shadcn由来componentは src/components/ui に置く.
- 記事本文HTMLはRust生成物だけを埋め込む.
