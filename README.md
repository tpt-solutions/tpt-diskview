# tpt-diskview

A fast, modern disk space analyzer — a TreeSize / WinDirStat alternative built with Rust and Tauri.

## Features

- **Blazing fast scanning** — parallel filesystem traversal in Rust; scan multi-TB volumes in seconds
- **Low memory usage** — incremental/streaming results, no need to hold entire tree in RAM
- **Three visualizations** — treemap, sunburst, and sortable tree list
- **Smart cleanup** — detect temp files, duplicate files, and stale Docker volumes
- **Cross-platform** — Windows and Linux
- **Zero telemetry** — nothing phones home, ever
- **Free & open source** — MIT OR Apache-2.0

## Installation

See [Releases](https://github.com/tpt-solutions/tpt-diskview/releases) for pre-built binaries.

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (LTS)
- [pnpm](https://pnpm.io/)
- [Tauri CLI](https://tauri.app/v2/guide/cli/)

### Build

```bash
pnpm install
pnpm tauri build
```

### Development

```bash
pnpm install
pnpm tauri dev
```

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.
