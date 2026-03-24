# Installation

## Pre-built binaries

Download the latest release from [GitHub Releases](https://github.com/mbaneshi/nexus/releases).

Binaries are available for:
- **macOS** — Apple Silicon (aarch64) and Intel (x86_64)
- **Linux** — x86_64

```bash
# macOS Apple Silicon
curl -sL https://github.com/mbaneshi/nexus/releases/latest/download/nexus-latest-aarch64-apple-darwin.tar.gz | tar xz
sudo mv nexus /usr/local/bin/
```

## Build from source

**Requirements:** Rust 1.85+ (edition 2024)

```bash
git clone https://github.com/mbaneshi/nexus.git
cd nexus
cargo build --release
# Binary at target/release/nexus
```

## Verify installation

```bash
nexus --help
```
