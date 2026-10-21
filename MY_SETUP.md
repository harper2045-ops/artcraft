# My ArtCraft fork

Personal fork of [storytold/artcraft](https://github.com/storytold/artcraft) for my own designs.

## Remotes and branches
- `origin` is my fork (harper2045-ops/artcraft) and `upstream` is storytold/artcraft.
- Do my work on the `my-designs` branch and leave `main` matching upstream.
- To pull in upstream changes:
  ```bash
  git fetch upstream && git checkout main && git merge upstream/main && git checkout my-designs && git merge main
  ```

## Toolchain on this Mac
- Rust (Homebrew), Node 26, tauri-cli 2.10.0 (installed with `cargo install`)

## Run it
```bash
./script/artcraft/unix_dev.sh
```
- Rust check without a database: `SQLX_OFFLINE=true cargo check -p artcraft`

## Where design changes go
- Colors, fonts and theme: `frontend/apps/artcraft/tailwind.config.ts`
- App UI (pages and panels): `frontend/apps/artcraft/app/`
- Shared UI components: `frontend/libs/components/`
- Icons: `frontend/libs/icons/`
- App name, window and icons: `crates/desktop/artcraft/` (Tauri config)
- Read `AGENTS.md` for the project's code style rules.

## Warning: the app's data folder is this repo
The app keeps its data in `~/Artcraft` (`DEFAULT_DATA_DIR` in
`crates/desktop/artcraft/src/core/state/data_dir/app_data_root.rs`). Mac folder names
ignore case, so that is this repo folder. Its `credentials/`, `state/`, `downloads/`
and other folders are listed in `.git/info/exclude` so they never get committed.
Cleaner fix: move the repo to a folder with a different name (for example `~/artcraft-src`).
