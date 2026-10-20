# moon-sandbox

Rust workspace for the shop API, its internal ERP surface, and a scratch app for
experiments. The workspace runs through **[moon](https://moonrepo.dev)**, not
through raw `cargo` commands: each project declares its tasks in `moon.yml`, and
moon handles the project graph, task caching, and `cargo watch` processes.

## Projects

| Project     | Path            | What it is                                                                          |
| ----------- | --------------- | ----------------------------------------------------------------------------------- |
| `axum-app`  | `apps/axum-app` | Axum API for the shop (`/api`) and internal ERP (`/api/internal`)                   |
| `rust-app`  | `apps/rust-app` | Scratch app; `src/playground.rs` is the place to test snippets                      |
| `migration` | `migration`     | SeaORM migrator crate — see [Migrations](#migrations) for why it is not a moon task |

## Prerequisites

- **Rust 1.85+** — edition 2024; tested with `1.97.1`
- **Moon 2.x** — tested with `2.4.6` (`moon --version`)
- **PostgreSQL** reachable through `DATABASE_URL`
- `cargo-watch` and `sea-orm-cli` — moon installs both from `.moon/toolchains.yml`

## Quick start

From the repository root:

```powershell
Copy-Item .env.example .env    # then fill in DATABASE_URL and COOKIE_KEY
cargo run -p migration -- up   # create tables
moon run axum-app:dev          # start the API on http://localhost:3002
```

## First-time setup

1. Copy the environment template and edit it:

   ```powershell
   Copy-Item .env.example .env
   ```

   At minimum set `DATABASE_URL`. `COOKIE_KEY` must be at least 64 characters;
   `HOST` / `PORT` default to `0.0.0.0:3002`, and `CORS_ORIGINS` is a
   comma-separated list of allowed browser origins.

2. Start PostgreSQL, then apply the identity-table migration (see
   [Migrations](#migrations)).

3. Start a project with `moon run <project>:dev`. The `dev` task is
   `cargo watch -x run`, so saving a source file restarts the process.

## Everyday commands

Run everything from the repository root.

| Command                   | What it does                                               |
| ------------------------- | ---------------------------------------------------------- |
| `moon run axum-app:dev`   | Watch and run the API (`GET /health` for a liveness check) |
| `moon run rust-app:dev`   | Watch and run the playground app                           |
| `moon run :check`         | `cargo check` across every project                         |
| `moon run :test`          | `cargo test` across every project                          |
| `moon run axum-app:build` | Release build of a single project                          |
| `moon query projects`     | List the projects moon discovered                          |

The `:<task>` form runs that task in all projects; `<project>:<task>` runs it in
one. Tasks themselves live in `apps/<project>/moon.yml` — edit those files to
change a command, and prefer moon over `cargo` so caching and affected-only runs
stay correct.

## Playground

`apps/rust-app/src/playground.rs` exposes `pub async fn run()`, which `main`
calls first inside the Tokio runtime, so `.await` works without extra setup:

```rust
pub async fn run() {
    println!("playground: start");

    // Write experiments here.

    println!("playground: done");
}
```

Edit the body, then:

```powershell
moon run rust-app:dev
```

The `syntax` demo runs after the playground; comment out the
`syntax::r#async::tokio::demo::run().await;` call in
`apps/rust-app/src/main.rs` if you want playground output only.

## Migrations

The `migration` crate sits at the repository root, outside the `apps/*` and
`packages/*` globs in `.moon/workspace.yml`, so moon does not manage it yet. Run
it through cargo:

```powershell
cargo run -p migration -- up        # apply pending migrations
cargo run -p migration -- status    # list applied and pending migrations
cargo run -p migration -- down      # roll back the last batch
cargo run -p migration -- generate <name>   # scaffold a new migration file
```

This is the one intentional exception to the moon-only workflow.

## Configuration

| File                   | Purpose                                             |
| ---------------------- | --------------------------------------------------- |
| `.moon/workspace.yml`  | Projects (`apps/*`, `packages/*`) and VCS settings  |
| `.moon/toolchains.yml` | Rust toolchain plus `cargo-watch` and `sea-orm-cli` |
| `apps/*/moon.yml`      | Per-project tasks: `dev`, `build`, `check`, `test`  |
| `.env`                 | Local secrets, copied from `.env.example`           |
