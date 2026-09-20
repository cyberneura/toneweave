# AGENTS.md

Guide for coding agents working in this repository. User-facing docs are in `README.md`.

## Stack

- Tauri 2 (Rust) + Svelte 5 + Vite + Tailwind CSS 4, TypeScript
- pnpm 11 (`packageManager` in `package.json`)
- LLM: OpenAI Chat Completions with strict JSON Schema structured output

## Layout

```
src/
  App.svelte                 Whole UI: toolbar, three panes, status bar, dialogs
  components/ReplyCard.svelte
  lib/api.ts                 Tauri command wrappers, types, browser-preview fallbacks
  app.css                    Color tokens (light / dark) and base styles
src-tauri/src/
  main.rs                    Tauri commands and entry point (GUI or CLI)
  config.rs                  config.yaml loading, override command, validation
  ai.rs                      Prompt / request body, response parsing
  draft.rs                   Draft autosave (draft.json, atomic write)
  results.rs                 Saves generated replies to results/
  cli.rs                     `toneweave reply ...`
resources/app-icons/         Icon masters
scripts/release.sh           `pnpm release`
scripts/release-decide.sh    Release gate used by the workflow
.github/workflows/release.yml
```

User data lives in `~/.config/toneweave/`: `config.yaml`, `draft.json`, `results/`.

## Commands

```bash
pnpm tauri dev
pnpm check     # svelte-check, must be 0 errors / 0 warnings
pnpm test      # cargo test
pnpm build     # frontend only
```

Run `pnpm check` and `pnpm test` after changes. Restart `pnpm tauri dev` after changing Rust code or `tauri.conf.json`.

## UI rules

- The app must look like a desktop app (VS Code, Slack, Numbers), not a website: toolbar, edge-to-edge panes, pane headers, status bar, system font, compact controls. No hero text, marketing copy or decorative illustrations.
- Style with Tailwind utilities. Colors come from the tokens in `app.css` (`bg-window`, `bg-pane`, `border-line`, `text-muted`, `bg-accent`, `text-on-accent`, `bg-scrim`, ...); do not hard-code colors. Every token has a dark-mode value.
- Keep `font-size` on `body`, not `:root`. Tailwind spacing is in `rem`, so a smaller root font shrinks every margin, height and padding.
- The toolbar is 40px (`h-10`) and starts at `pl-20` to leave room for the macOS traffic lights (`trafficLightPosition` in `tauri.conf.json`). Change both together.
- Drag regions need `data-tauri-drag-region` and the `core:window:allow-start-dragging` permission.
- Add `data-annotate="..."` to every interactive element for E2E tests.
- Windows builds ship too. Write shortcut labels through the `keys` map in `App.svelte`, never as a bare `⌘`.
- Text selection is disabled app-wide. Add the `selectable` class to content users may copy.

## Behavior to preserve

- **Example config**: on GUI launch, `config.yaml` is created from `config.example.yaml` (embedded with `include_str!`) only if it does not exist (`create_new`, mode 0600). Keep `config.example.yaml` loadable: a test loads it.
- **API key gate**: when `configured` is false in the desktop app, a blocking dialog is shown and generation and shortcuts are disabled. The browser preview (`pnpm dev`) is never blocked.
- **Presets and decorations are sent by name/title only.** Rust resolves the prompt text from the config and rejects unknown names, so the UI cannot inject arbitrary prompt text through these fields.
- **Decoration groups** are exclusive in the UI and validated again in `ai.rs`, because the CLI bypasses the UI and can ask for a conflicting combination.
- **Reply count** (`n`, 1–3) overrides `general.default_n` per request and is re-validated in Rust.
- **Draft autosave**: form state is written through Rust (`draft.rs`) 400 ms after the last change, via temp file + rename, mode 0600. `localStorage` is not used because WebKit flushes it lazily and loses recent edits on a crash. The draft includes the source email.
- `ai.api_key` is never serialized to the frontend (`#[serde(skip_serializing)]`).
- `source_email` is passed to the model as quoted data, never as instructions.

## LLM model names

The default model is `gpt-5.6-terra`. Defaults are defined in both `src-tauri/src/config.rs` and `src/lib/api.ts`, and the example in `config.example.yaml`; update all three together. The source of truth for current model names is `cyberneura/llmlib` `settings.py`.

GPT-5-family models reject `temperature` and `max_tokens`. Do not add them to the request in `ai.rs`.

## App icon

Masters are in `resources/app-icons/`: `toneweave-mac-icon.png` (10% margin, macOS only) and `toneweave-favicon.png` (no margin, everything else). `pnpm tauri icon` overwrites all of `src-tauri/icons/`, so regenerate in this order:

```bash
ICONS=src-tauri/icons
pnpm tauri icon "$PWD/resources/app-icons/toneweave-mac-icon.png"
cp $ICONS/icon.icns /tmp/icon.icns
pnpm tauri icon "$PWD/resources/app-icons/toneweave-favicon.png"
cp /tmp/icon.icns $ICONS/icon.icns
rm -rf $ICONS/android $ICONS/ios
```

## Release

- The version in `src-tauri/tauri.conf.json` on `main` decides the release. `release.yml` runs on every push to `main` and only builds when that version is unreleased and newer than the latest release.
- Bump the version only with `pnpm release` as its own commit. Never mix a version bump into a feature change: merging it publishes a release.
- CI signs with Developer ID and notarizes the macOS build. The build fails if any of these repository secrets is missing: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`.
- Keep `signingIdentity: "-"` in `tauri.conf.json` (ad-hoc for local builds; CI overrides it through env).
- The Homebrew cask is `Casks/toneweave.rb` in `cyberneura/homebrew-tap`. The tap's own workflow updates its `version` and `sha256` every hour from the latest release. This repository never pushes to the tap.
- Never use "Re-run failed jobs" on an older run once a newer run for the same version exists: the old run would publish its own (older) commit's build. Push a fix, or bump the version, and let a fresh run do it.
- Never create the `v<version>` tag yourself. GitHub creates it when the draft is published; if the tag already exists, the release is published as `untagged-<hash>` instead and the `/download/v<version>/...` URLs break. `release-decide.sh` refuses to release while such a tag exists.
- All `uses:` in the workflow are pinned to commit SHAs. Keep them pinned.
