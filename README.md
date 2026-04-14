# S3V

S3-compatible storage client built with [Tauri v2](https://tauri.app), [SvelteKit](https://svelte.dev), and [Tailwind CSS](https://tailwindcss.com).

Supports **AWS S3**, **RustFS(MinIO)**, **Cloudflare R2**, and any S3-compatible provider.

[![Release](https://github.com/aciddust/s3v/actions/workflows/release.yml/badge.svg)](https://github.com/aciddust/s3v/actions/workflows/release.yml)

## Features

- Multi-profile connection management (tabs)
- Dual panel file browser with independent navigation
- Drag-and-drop file copy/move between panels
- Upload, download, rename, delete, create folder
- Presigned URL sharing (clipboard)
- Breadcrumb navigation with bookmarks
- Infinite scroll pagination (1000 items/page)
- Transfer queue with progress tracking
- Keyboard shortcuts
- Context menu actions
- Search / filter files
- Feedback form (built-in)

## Install

### macOS (Homebrew)

```bash
brew tap aciddust/tap
brew install --cask s3v
```

Update:

```bash
brew update
brew upgrade --cask s3v
```

### Manual Download

Download the latest release from the [Releases](https://github.com/aciddust/s3v/releases) page.

| Platform | File |
| --- | --- |
| macOS (Universal) | `s3v_*_universal.dmg` |
| Windows x64 | `s3v_*_x64-setup.exe` |
| Windows ARM64 | `s3v_*_arm64-setup.exe` |
| Linux x64 (AppImage) | `s3v_*_amd64.AppImage` |
| Linux x64 (deb) | `s3v_*_amd64.deb` |

## Prerequisites

- [Bun](https://bun.sh) >= 1.3
- [Rust](https://rustup.rs) >= 1.77
- [mold](https://github.com/rui314/mold) (Linux, faster linker) or [lld](https://lld.llvm.org) (macOS/Windows)
- Tauri v2 system dependencies ([setup guide](https://v2.tauri.app/start/prerequisites/))

## Quick Start

```bash
# Install dependencies
bun run install:all

# Start development server
bun run dev:tauri
```

## Available Commands

| Command | Description |
| --- | --- |
| `bun run dev:tauri` | Start Tauri dev (frontend + backend hot-reload) |
| `bun run check:all` | Type-check Svelte + Rust |
| `bun run fmt` | Format code (oxfmt + Prettier for Svelte + cargo fmt) |
| `bun run build` | Build frontend only |
| `bun run bundle` | Build distributable app (dmg/msi/deb) |
| `bun run bundle:debug` | Build debug bundle (faster, not optimized) |
| `bun run logo` | Generate app icons from `static/s3v-logo.png` |
| `bun run deploy` | Tag current version and push to trigger release |
| `bun run deploy:dry` | Show what version would be released |
| `bun run rustfs` | Start local RustFS for testing |
| `bun run rustfs:down` | Stop RustFS |
| `bun run rustfs:clean` | Stop RustFS and delete all data |
| `bun run clean` | Remove build artifacts |

## Local Testing with RustFS

Tested with [RustFS](https://rustfs.com) — a Rust-native S3-compatible object storage.

```bash
# Start RustFS + seed test data
bun run rustfs
```

Creates three test buckets:

- `test-small` - 10 files
- `test-large` - 2500 files (pagination test)
- `test-nested` - folder hierarchy

Connect in S3V:

| Field | Value |
| --- | --- |
| Provider | rustfs |
| Endpoint | `http://localhost:9000` |
| Region | `us-east-1` |
| Access Key | `rustfsadmin` |
| Secret Key | `rustfsadmin` |
| Path Style | on |

Web Console: [http://localhost:9001](http://localhost:9001)

## Project Structure

```text
src/                        # SvelteKit frontend
  lib/
    api/                    # Tauri invoke wrappers
    components/             # Svelte components
    stores/                 # Svelte 5 rune stores
    constants.ts            # App constants
  routes/                   # SvelteKit pages
src-tauri/                  # Rust backend
  src/
    profile/                # Connection profile management
    s3/                     # S3 operations (list, copy, move, delete...)
    transfer/               # Upload/download engine
    feedback.rs             # Feedback API
    logger.rs               # Log event system
```

## Release

Releases are built automatically via GitHub Actions on tag push.

```bash
bun run deploy:dry    # Check version
bun run deploy        # Tag + push → builds macOS Universal, Windows x64/ARM64, Linux x64
```

Artifacts: `.dmg` (macOS), `.exe` NSIS installer (Windows), `.AppImage` / `.deb` (Linux)

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop runtime | Tauri v2 |
| Frontend | SvelteKit + Svelte 5 |
| Styling | Tailwind CSS + shadcn-svelte |
| Backend | Rust (tokio, aws-sdk-s3) |
| S3 API | AWS SDK for Rust |
| Package manager | Bun |
| Linter | oxlint + tsgolint (type-aware) |
| Formatter | oxfmt (TS/CSS), Prettier (Svelte) |
| Clipboard | tauri-plugin-clipboard-manager |
| HTTP | reqwest (Rust-side, CORS-free) |

## License

MIT with [Commons Clause](https://commonsclause.com/) — free to use, modify, and share, but not for commercial sale. See [LICENSE](LICENSE).
