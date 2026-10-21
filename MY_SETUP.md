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

## App data lives outside the repo
The app keeps its data (credentials, downloads, settings) in `~/Artcraft`, which is set by
`DEFAULT_DATA_DIR` in `crates/desktop/artcraft/src/core/state/data_dir/app_data_root.rs`.
Mac folder names ignore case, so the repo lives in `~/artcraft-src` to keep code and data apart.
Don't clone the repo into `~/ArtCraft`.

## fal for 3D and video (work in progress)
- Image-to-3D (Hunyuan 3D 3.0/2.0) and image-to-video (Kling, Veo) can run on fal with my own key.
- **3D results don't save to the library yet.** fal makes the model, but ArtCraft's upload server
  rejects the `.glb` ("unknown mimetype"). For now, grab the file from `~/Artcraft/temp/` or the fal
  link and import it into 3D Stage by hand. Next step: try a different upload route for 3D files.
- **The fal video path hasn't been tested yet.**
- "Image to 3D World" (splats) still runs on ArtCraft credits only.
