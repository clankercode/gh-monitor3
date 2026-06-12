# gh-monitor3: GitHub Activity Overlay

## Overview

A lightweight, native transparent overlay app for monitoring GitHub repository activity at a glance. Shows a compressed timeline of events grouped by repo, with smooth animations for new/updated entries. Click+drag to move, click to open GitHub links. Cross-platform: Linux, macOS, Windows.

## Tech Stack Decision (IGC Evaluation)

**Decision: Rust + winit + wgpu + glyphon**

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Language | Rust | Memory safety, excellent ecosystem, cross-compilation, Cargo CI/CD |
| Windowing | winit 0.30 | Transparent windows, click-through (`set_cursor_hittest`), always-on-top |
| GPU Rendering | wgpu 29 | `CompositeAlphaMode::PostMultiplied` for transparency, cross-platform GPU |
| Text | glyphon 0.11 | wgpu-native text, cosmic-text shaping, font loading, colored text |
| GitHub API | octocrab 0.53 | Typed events, ETag support, pagination |
| Async Runtime | tokio | Industry standard, reqwest/octocrab dependency |
| Config | toml + serde | Human-readable config files |
| CLI | clap | Argument parsing |
| Logging | tracing | Structured logging |
| Testing | insta (snapshots) | Snapshot tests for renderer output |

### Known Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Windows transparency bug (winit #2502) | Test on Windows CI; fallback to opaque background with subtle border |
| Wayland AlwaysOnTop unsupported | Use layer-shell protocol if available; document limitation |
| X11 compositor dependency | Detect compositor; graceful fallback to opaque mode |
| GPU testing in CI | Separate logic tests (no GPU) from render tests (manual/optional) |

## Architecture

```
┌─────────────────────────────────────────────────┐
│                   Application                    │
├──────────┬──────────┬──────────┬────────────────┤
│  Window  │ Renderer │   API    │    Timeline    │
│ Manager  │ (wgpu)   │ Client   │   Engine       │
│ (winit)  │          │ (octocrab)│               │
├──────────┼──────────┼──────────┼────────────────┤
│ Platform │ glyphon  │ Events   │  Grouping      │
│ Specific │ Animati- │ Polling  │  Compression   │
│ Code     │ on Engine│ ETags    │  Time Humanize │
└──────────┴──────────┴──────────┴────────────────┘
```

### Module Breakdown

```
src/
├── main.rs              # Entry point, app lifecycle
├── config.rs            # Configuration (repos, orgs, token, polling)
├── github/
│   ├── mod.rs
│   ├── client.rs        # Octocrab wrapper with ETag caching
│   ├── events.rs        # Event type definitions and parsing
│   └── polling.rs       # Polling loop with conditional requests
├── timeline/
│   ├── mod.rs
│   ├── grouping.rs      # Group events by repo + type + time window
│   ├── compression.rs   # Compress similar consecutive events
│   ├── humanize.rs      # "1-3 hrs ago" time formatting
│   └── state.rs         # Timeline state management
├── render/
│   ├── mod.rs
│   ├── pipeline.rs      # wgpu render pipeline setup
│   ├── shapes.rs        # Rounded rectangles, badges, dividers
│   ├── text.rs          # glyphon text rendering wrapper
│   ├── timeline_view.rs # Timeline layout and rendering
│   └── theme.rs         # Colors, fonts, sizing
├── animation/
│   ├── mod.rs
│   ├── tween.rs         # Easing functions and tweening
│   ├── opacity.rs       # Fade in/out animations
│   └── manager.rs       # Animation state management
├── window/
│   ├── mod.rs
│   ├── overlay.rs       # Transparent window creation and management
│   ├── input.rs         # Click, drag, hover handling
│   └── platform/
│       ├── mod.rs
│       ├── linux.rs     # X11/Wayland specific code
│       ├── macos.rs     # NSWindow specific code
│       └── windows.rs   # Win32 specific code
├── app.rs               # Main application state and event loop
└── error.rs             # Error types
```

## Features

### Core Features
1. **Transparent overlay window** - Always on top, fades opaque on hover
2. **Click+drag to move** the overlay anywhere on screen
3. **Click to open links** - Opens GitHub repos/PRs in browser
4. **GitHub event monitoring** - Tracks PRs, issues, pushes, releases, forks, repo creation
5. **Timeline with grouped events** - Events grouped by repo, showing (event_type, count) pairs
6. **Humanized time ranges** - "1-3 hrs ago", "just now", "2-5 days ago"
7. **Organization monitoring** - Track all repos in a GitHub org
8. **ETag-based polling** - Efficient conditional requests (304 = free)

### Visual Features
9. **Fade-in animation** for new timeline elements (100% opacity → fade out)
10. **Subtle pulse animation** for updated elements (new PR in existing group)
11. **Compressed timeline** - Similar events collapse into single entries
12. **Rare events stand out** - Repo creation, releases get special visual treatment
13. **Dark theme** with color-coded event type badges

### Event Type Visual Design
| Event Type | Badge Color | Priority |
|------------|-------------|----------|
| CreateEvent (repo) | Gold/Yellow | High (standalone) |
| ReleaseEvent | Purple | High (standalone) |
| PullRequestEvent | Green | Normal (grouped) |
| IssuesEvent | Blue | Normal (grouped) |
| PushEvent | Gray | Normal (grouped) |
| ForkEvent | Orange | Normal (grouped) |
| Others | Dim | Low (grouped) |

## Implementation Plan

### Phase 1: Foundation (Tasks 1-3)
- Project scaffolding, Cargo.toml, CI/CD setup
- Config system (TOML, CLI args)
- GitHub API client with ETag support

### Phase 2: Core Logic (Tasks 4-6)
- Event grouping and compression
- Time humanization
- Timeline state management

### Phase 3: Window & Rendering (Tasks 7-10)
- Transparent window with winit + wgpu
- 2D shape rendering (rounded rects, badges)
- Text rendering with glyphon
- Timeline layout engine

### Phase 4: Interaction & Animation (Tasks 11-13)
- Click, drag, hover input handling
- Animation system (tweening, opacity)
- Platform-specific click-through code

### Phase 5: Integration & Polish (Tasks 14-16)
- Full app integration
- Error handling and resilience
- Performance optimization

### Phase 6: Testing & Release (Tasks 17-19)
- Comprehensive test suite
- Snapshot tests for renderer
- Cross-platform CI builds and releases

---

## SUMMARY

**What:** A native transparent overlay for monitoring GitHub activity in real-time. Shows a compressed timeline of events grouped by repository with smooth animations.

**Tech:** Rust + winit (windowing) + wgpu (GPU rendering) + glyphon (text) + octocrab (GitHub API). Pure Rust stack, no native dependencies beyond GPU drivers.

**Architecture:** Modular design with separate crates/paths for window management, GPU rendering, GitHub API client, timeline engine (grouping/compression), and animation system. Event-driven architecture with tokio async runtime.

**Key decisions:**
- winit for cross-platform windowing (transparency, click-through, always-on-top)
- wgpu with `CompositeAlphaMode::PostMultiplied` for transparent rendering
- glyphon for GPU-accelerated text with cosmic-text shaping
- ETag-based polling for efficient GitHub API usage (304 responses are free)
- Timeline compression: group by repo + event type + time window, show (type, count) pairs
- Rare events (repo creation, releases) get standalone visual treatment

**Risks:** Windows transparency has known winit bug (#2502); Wayland AlwaysOnTop unsupported; X11 requires compositor. All have documented mitigations.
