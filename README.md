# ROBS - Rust OBS Studio

A complete rewrite of OBS Studio in Rust, designed for modern streaming workflows with multi-destination streaming, unified chat aggregation, and a dockable interface.

## Features

### Core Streaming
- **Multi-destination streaming** - Stream to Twitch, YouTube, Facebook, and other platforms simultaneously
- **RTMP output** with automatic reconnection and configurable retry logic
- **File recording** with support for MP4, MKV, FLV, and MOV containers
- **x264 encoder** with full preset support (ultrafast through veryslow) and CBR/VBR/CRF rate control modes
- **Multi-track audio** with per-source volume control and mixing

### User Interface
- **Dockable panel system** with resizable, toggleable panels
- **Preview window** with live indicator during streaming
- **Sources panel** for managing capture sources with visibility toggles
- **Scenes panel** for quick scene switching
- **Audio mixer** with per-channel volume sliders, mute buttons, and real-time level meters
- **Unified chat window** aggregating messages from multiple platforms
- **Real-time stats** showing duration, bitrate, frame rate, and dropped frames
- **Comprehensive settings** with tabs for General, Video, Audio, Hotkeys, Streaming, and Outputs

### Chat Integration
- **Twitch IRC** integration for real-time chat messages
- **YouTube Live Chat** support
- **Unified message display** with platform-specific color coding
- **Per-platform filtering** to view chat from specific sources

### Profiles & Settings
- **Profile system** with save/load/duplicate functionality
- **TOML-based serialization** for human-readable configuration
- **Video configuration** including resolution, FPS, and downscale filter
- **Audio configuration** with sample rate and channel settings
- **Streaming configuration** with server, bitrate, encoder, and keyframe settings

## Architecture

ROBS is organized as a Rust workspace with modular crates:

| Crate | Purpose |
|-------|---------|
| `robs-core` | Core types, traits, pipeline, event system, error handling |
| `robs-video` | Video processing pipeline and frame handling |
| `robs-audio` | Audio sources, mixing, and processing |
| `robs-encoding` | Encoder implementations (x264 with extensible trait system) |
| `robs-outputs` | RTMP streaming, file recording, multi-destination output |
| `robs-sources` | Capture sources (window, monitor, game, test pattern) |
| `robs-ui` | egui-based graphical user interface |
| `robs-plugins` | Plugin architecture with dynamic library loading |
| `robs-profiles` | Profile management and settings persistence |
| `robs-chat` | Multi-platform chat aggregation (Twitch, YouTube) |
| `robs` | Main application binary |

## Building

### Prerequisites

- Rust 1.75+ with the MSVC toolchain (`stable-x86_64-pc-windows-msvc`)
- Windows 10 or later

### Setup

Install the MSVC toolchain:

```powershell
rustup toolchain install stable-x86_64-pc-windows-msvc
rustup default stable-x86_64-pc-windows-msvc
```

### Build

```powershell
cargo build --release
```

The compiled binary will be at `target\x86_64-pc-windows-msvc\release\robs.exe`.

### Run

```powershell
cargo run
```

## Current Status

This is an early-stage project with a functional UI and core architecture in place. The following components are implemented:

- Complete UI with all major panels and settings
- Profile management system
- Chat aggregation framework (simulated messages)
- Encoder trait system with x264 preset configuration
- RTMP output structure with multi-destination support
- Audio mixer with per-channel controls
- Plugin loading architecture

### Not Yet Implemented

- Actual RTMP protocol implementation (handshake is stubbed)
- Real video encoding (x264 produces empty packets)
- Hardware encoder support (NVENC, QSV)
- Actual capture implementations (window, monitor, game)
- Audio capture via WASAPI
- FFmpeg integration for recording

## Design Goals

- **Memory safety** through Rust's ownership system
- **Concurrency** with async/await and lock-free data structures where possible
- **Extensibility** through trait-based plugin architecture
- **Cross-platform** potential (currently Windows-focused)
- **Performance** with LTO and optimized release builds

## License

GPL-3.0
