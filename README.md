# Toneweave

Desktop app that drafts email replies with an LLM. Paste the email you received, say how you want to reply, and Toneweave writes reply drafts in your voice. It never sends email.

## Install

> No release has been published yet. Until the first one, build from source (see [Development](#development)). The instructions below apply from the first release on.

macOS (Apple Silicon / Intel):

```bash
brew install --cask cyberneura/tap/toneweave
```

Or download the `.dmg` (macOS) or the `.exe` installer (Windows) from [Releases](https://github.com/cyberneura/toneweave/releases).

## Setup

Toneweave needs an OpenAI API key. The app blocks generation until one is set. (With `ai.provider: openai-compatible` the key may be empty, for a local model that does not authenticate.)

On first launch the app creates `~/.config/toneweave/config.yaml` from [`config.example.yaml`](config.example.yaml). An existing file is never overwritten. Set `ai.api_key` in it, then click **Reload** in the app.

`OPENAI_API_KEY` also works, but apps opened from Finder do not read your shell environment.

## Usage

1. Paste the email into **Original email** (toolbar **Paste** or ⌘V / Ctrl+V).
2. In **Options**, write a direction (e.g. 「やんわり断りつつ、次の機会につなげたい」), then choose the tone, number of replies (1–3), decorations and greeting.
3. Press **Generate** (⌘↵ / Ctrl+Enter). Copy the reply you like.

| macOS | Windows | Action |
|-------|---------|--------|
| ⌘↵ | Ctrl+Enter | Generate replies |
| ⌘, | Ctrl+, | Open settings |
| Esc | Esc | Close dialogs |

The current draft is autosaved to `~/.config/toneweave/draft.json` and restored on the next launch, including after a crash. **New → Clear** deletes it. Generated replies are also saved to `~/.config/toneweave/results/`.

The source email is sent to the configured AI provider when you generate.

## Configuration

`~/.config/toneweave/config.yaml` — see [`config.example.yaml`](config.example.yaml) for every option.

| Key | Description |
|-----|-------------|
| `ai.provider` | `openai` or `openai-compatible` |
| `ai.api_key` | API key (falls back to `OPENAI_API_KEY`) |
| `ai.model` | Model name (default `gpt-5.6-terra`) |
| `ai.base_url` | API base URL. HTTPS, or HTTP for localhost only |
| `general.default_n` | Initial number of replies (1–3) |
| `general.clipboard_auto_paste` | On launch, paste the clipboard into the source field (only with a configured key, and only when the field is empty) |
| `general.auto_copy_result` | Copy the first reply after generating |
| `greeting.*` | Default greeting / keigo switch and its opening and closing guidance |
| `preset_prompts` | Tone presets shown in **Tone** |
| `decorations` | Toggle buttons that add fixed instructions to the prompt |
| `config_override_command` | Command whose stdout (YAML) is merged over the file, e.g. `op read ...` |

### Decorations

Each decoration becomes a toggle button under **Decorations**. Every enabled `context` is added to the prompt, and it takes precedence over the tone preset and the greeting guidance. Decorations that share a `group` are mutually exclusive.

```yaml
decorations:
  - title: "お世話: 会社A"
    context: "「いつもお世話になっております。株式会社A 山田です。」を追加"
    group: opening
  - title: "お世話: 会社B"
    context: "「いつもお世話になっております。株式会社B 山田です。」を追加"
    group: opening
```

## CLI

```bash
pbpaste | toneweave reply -d "やんわり断る" -p 丁寧に -g on -D "お世話: 会社A"
toneweave reply --clipboard
toneweave --help
```

With no arguments, the desktop app opens. The CLI is the executable inside the app bundle (`Toneweave.app/Contents/MacOS/`); it is not added to `PATH` automatically.

## Development

Requirements: Node.js 22+, pnpm 11, Rust stable, Xcode Command Line Tools (macOS).

```bash
pnpm install
pnpm tauri dev      # run the app
pnpm check          # svelte-check
pnpm test           # cargo test
pnpm tauri build    # local build (ad-hoc signed)
```

## Release

The version in `src-tauri/tauri.conf.json` on `main` decides the release. When an unreleased version reaches `main`, GitHub Actions tests, builds a signed and notarized universal `.dmg` and a Windows `.exe`, and publishes a GitHub Release. Once `Casks/toneweave.rb` exists in the [Homebrew tap](https://github.com/cyberneura/homebrew-tap), the tap picks up each new release within an hour.

```bash
pnpm release          # patch (default)
pnpm release minor
pnpm release major
```

`pnpm release` bumps the version, commits, pushes to `main` and watches the workflow.
