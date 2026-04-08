# S3V

S3-compatible storage client built with [Tauri v2](https://tauri.app), [SvelteKit](https://svelte.dev), and [Tailwind CSS](https://tailwindcss.com).

Supports **AWS S3**, **MinIO**, **Cloudflare R2**, and any S3-compatible provider.

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

## Prerequisites

- [Node.js](https://nodejs.org) >= 18
- [Rust](https://rustup.rs) >= 1.77
- Tauri v2 system dependencies ([setup guide](https://v2.tauri.app/start/prerequisites/))

## Quick Start

```bash
# Install dependencies
make install

# Start development server
make dev
```

## Available Commands

```bash
make help
```

| Command | Description |
| --- | --- |
| `make dev` | Start Tauri dev (frontend + backend hot-reload) |
| `make check` | Type-check Svelte + Rust |
| `make fmt` | Format code (Prettier + cargo fmt) |
| `make build` | Build frontend only |
| `make bundle` | Build distributable app (dmg/msi/deb) |
| `make bundle-debug` | Build debug bundle (faster, not optimized) |
| `make logo` | Generate app icons from `static/s3v-logo.png` |
| `make deploy` | Tag current version and push to trigger release |
| `make deploy-dry` | Show what version would be released |
| `make minio` | Start local MinIO for testing |
| `make minio-down` | Stop MinIO |
| `make minio-clean` | Stop MinIO and delete all data |
| `make clean` | Remove build artifacts |

## Local Testing with MinIO

```bash
# Start MinIO + seed test data
make minio
```

Creates three test buckets:

- `test-small` - 10 files
- `test-large` - 2500 files (pagination test)
- `test-nested` - folder hierarchy

Connect in S3V:

| Field | Value |
| --- | --- |
| Provider | minio |
| Endpoint | `http://localhost:9000` |
| Region | `us-east-1` |
| Access Key | `minioadmin` |
| Secret Key | `minioadmin` |
| Path Style | on |

MinIO Web Console: [http://localhost:9001](http://localhost:9001)

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
make deploy-dry    # Check version
make deploy        # Tag + push → builds macOS Universal, Windows x64, Windows ARM64
```

Artifacts: `.dmg` (macOS), `.exe` NSIS installer (Windows)

## Tech Stack

| Layer | Technology |
| --- | --- |
| Desktop runtime | Tauri v2 |
| Frontend | SvelteKit + Svelte 5 |
| Styling | Tailwind CSS + shadcn-svelte |
| Backend | Rust (tokio, aws-sdk-s3) |
| S3 API | AWS SDK for Rust |
| Clipboard | tauri-plugin-clipboard-manager |
| HTTP | reqwest (Rust-side, CORS-free) |

## License

MIT with [Commons Clause](https://commonsclause.com/) — free to use, modify, and share, but not for commercial sale. See [LICENSE](LICENSE).
