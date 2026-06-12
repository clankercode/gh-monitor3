# gh-monitor3

A lightweight native transparent overlay for monitoring GitHub repository activity at a glance.

## Features
- Transparent overlay window (fades opaque on hover)
- Click+drag to move, click to open GitHub links
- Timeline with grouped/compressed events by repo
- Humanized time ranges ("1-3 hrs ago", "just now")
- Organization-wide monitoring
- ETag-based efficient polling (304 responses are free)
- Smooth animations for new/updated entries
- Cross-platform: Linux, macOS, Windows

## Installation

### From releases
Download the latest binary from [Releases](https://github.com/clankercode/gh-monitor3/releases).

### From source
```bash
cargo install --path .
```

## Usage

```bash
# Run with default config
gh-monitor3

# Run with a GitHub token (for higher rate limits)
gh-monitor3 --token ghp_your_token_here

# Generate default config file
gh-monitor3 --generate-config

# Use custom config path
gh-monitor3 --config /path/to/config.toml
```

## Configuration

Config file location: `~/.config/gh-monitor3/config.toml`

```toml
github_token = "ghp_..."
poll_interval_secs = 60

[[repos]]
owner = "rust-lang"
name = "rust"

[[repos]]
owner = "tokio-rs"
name = "tokio"

[orgs]
# List org names to monitor all their public repos
orgs = ["facebook", "google"]

[window]
width = 320
height = 480
opacity = 0.15
hover_opacity = 0.95

[theme]
font_size = 13.0
```

## Environment Variables
- `GITHUB_TOKEN` - GitHub personal access token (alternative to config)
- `RUST_LOG` - Log level (e.g., `info`, `debug`, `gh-monitor3=debug`)

## Development

```bash
# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy

# Build release
cargo build --release
```

## Architecture
- `src/github/` - GitHub API client with ETag caching and polling
- `src/timeline/` - Event grouping, compression, and time humanization
- `src/render/` - wgpu GPU pipeline with 2D shapes and glyphon text
- `src/animation/` - Tweening engine and opacity animations
- `src/window/` - Platform-specific overlay management

## License
MIT OR Apache-2.0
