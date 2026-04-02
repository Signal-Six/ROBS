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
- FFmpeg 6+ must be installed and available in your system PATH

### FFmpeg Requirement

ROBS requires a recent version of FFmpeg to be installed on your system. The encoding pipeline (x264, NVENC, AAC) depends on FFmpeg being available as a system command. Ensure `ffmpeg` is accessible from your command line before running ROBS.

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

This is an early-stage project with a functional UI and core architecture in place. The following major components are implemented:

- ✅ Complete UI with all panels (Sources, Scenes, Preview, Audio Mixer, Chat, Stats, Settings)
- ✅ Settings window with proper resizing and close behavior
- ✅ Recording path selector with file dialog and format selection
- ✅ Profile management system with TOML serialization
- ✅ Chat aggregation framework (simulated messages)
- ✅ FFmpeg H.264 software encoder with full preset support
- ✅ NVIDIA NVENC hardware encoder with auto-detection  
- ✅ AAC audio encoder with bitrate control
- ✅ Encoder factory with availability detection (FFmpeg, NVENC, AAC)
- ✅ Multi-destination streaming architecture
- ✅ Audio mixer with per-channel volume, mute, and meters
- ✅ Plugin loading architecture
- ✅ Recording start/stop with timestamped filenames

### Not Yet Implemented (High Priority)

1. **RTMP protocol implementation** - streaming currently stubbed
2. **Video capture sources** - window, monitor, game capture
3. **Audio capture** - WASAPI/device input
4. **FFmpeg encoding pipeline integration** for actual recording output
5. **Network streaming** - actual RTMP handshake and packet transmission
6. **Preview rendering** - live video display
7. **Scene composition** - source layout and blending

### Technical Status

The project successfully builds and runs with:
- MSVC toolchain support (Windows target)
- FFmpeg dependency detection on startup  
- NVENC hardware acceleration detection
- AAC audio encoding availability
- All UI controls functional except actual media capture/streaming

The codebase provides a solid foundation with proper architecture, modular crates, and trait-based extensibility. The remaining work focuses on integrating actual media capture and streaming capabilities.

## Design Goals

- **Memory safety** through Rust's ownership system
- **Concurrency** with async/await and lock-free data structures where possible
- **Extensibility** through trait-based plugin architecture
- **Cross-platform** potential (currently Windows-focused)
- **Performance** with LTO and optimized release builds

## License

GPL-3.0
