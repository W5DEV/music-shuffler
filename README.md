# 🎵 Music Shuffler

A fast, lightweight music player that creates instant random playlists from your music library. Built in Rust for maximum performance and reliability.

![Windows](https://img.shields.io/badge/Windows-0078D4?style=for-the-badge&logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![GitHub release](https://img.shields.io/github/v/release/W5DEV/music-shuffler?style=for-the-badge)
![GitHub Downloads](https://img.shields.io/github/downloads/W5DEV/music-shuffler/total?style=for-the-badge)

## ✨ Features

- 🎲 **Instant Random Playlists** - Generate 50 random songs in seconds
- ⚡ **Lightning Fast** - Optimized for huge music libraries (100GB+)
- 🔄 **Smart Caching** - First scan takes time, every launch after is instant
- 🎵 **Multi-Format Support** - MP3, FLAC, OGG, WAV, M4A, AAC
- 🖼️ **Album Art Display** - Beautiful album artwork when available
- 📊 **Real-time Progress** - Live progress bars and time tracking
- 💾 **Remembers Everything** - Your directory, preferences, and metadata
- 🎯 **Zero Configuration** - Just select your music folder and go

## 🚀 Quick Start

### Download & Run

1. **Download** the latest [release](https://github.com/W5DEV/music-shuffler/releases)
2. **Extract** `music-shuffler-windows.zip`
3. **Run** `music-shuffler.exe`
4. **Select** your music directory
5. **Click** "Generate Playlist"
6. **Enjoy** your music! 🎶

### First Launch Experience

```
┌─────────────────────────────────────────────────────────────┐
│                    🎵 Music Shuffler                        │
├────────────────────────────────────────────────────────────┤
│           Select a music directory to get started          │
│                                                            │
│  [Select Directory]  [Generate Playlist]                   │
├────────────────────────────────────────────────────────────┤
│ Playlist                │  Now Playing                     │
│                         │                                  │
│ ♪ Bohemian Rhapsody     │  [Album Art]                     │
│   Hotel California      │                                  │
│   Stairway to Heaven    │  Bohemian Rhapsody               │
│   Sweet Child O' Mine   │  Queen                           │
│   Free Bird             │  A Night at the Opera            │
│   ... (45 more songs)   │                                  │
│                         │  ████████░░░░ 2:34 / 5:55        │
│                         │        ⏮  ⏸  ⏭                  │
└─────────────────────────────────────────────────────────────┘
```

## 📋 System Requirements

- **OS:** Windows 10 or later (64-bit)
- **RAM:** 100MB (scales with library size)
- **Storage:** 25MB + cache space
- **Audio:** Any Windows-compatible audio device

## 🎯 Performance

| Library Size         | First Scan    | Subsequent Launches | Playlist Generation |
| -------------------- | ------------- | ------------------- | ------------------- |
| 1GB (500 songs)      | 5-10 seconds  | 0.1 seconds         | 0.1 seconds         |
| 10GB (5,000 songs)   | 15-30 seconds | 0.2 seconds         | 0.2 seconds         |
| 100GB (50,000 songs) | 30-60 seconds | 0.5 seconds         | 0.3 seconds         |

_Performance scales beautifully thanks to intelligent caching and parallel processing._

## 🎵 Supported Formats

| Format   | Metadata   | Album Art  | Playback Quality |
| -------- | ---------- | ---------- | ---------------- |
| **MP3**  | ✅ Full    | ✅ Yes     | High             |
| **FLAC** | ✅ Full    | ✅ Yes     | Lossless         |
| **OGG**  | ✅ Full    | ✅ Yes     | High             |
| **WAV**  | ⚠️ Limited | ❌ No      | Lossless         |
| **M4A**  | ✅ Full    | ✅ Yes     | High             |
| **AAC**  | ✅ Full    | ⚠️ Limited | High             |

## 🔧 How It Works

### Intelligent Scanning

1. **Parallel Discovery** - Uses all CPU cores to find music files
2. **Smart Filtering** - Only processes actual music files
3. **Metadata Extraction** - Reads title, artist, album, and duration
4. **Cache Building** - Saves everything for instant future access

### Smart Caching

- **File List Cache** - Remembers all discovered music files
- **Metadata Cache** - Stores extracted song information
- **Change Detection** - Only re-processes modified files
- **Cross-Session** - Cache persists between app launches

### Audio Engine

- **Rust-Native** - Built with the Rodio audio library
- **Low Latency** - Minimal delay between track changes
- **Robust Decoding** - Handles various encoding qualities
- **Error Recovery** - Gracefully skips corrupted files

## 🛠️ Troubleshooting

### Audio Issues

**No sound when playing:**

- Install [Visual C++ Redistributables](https://aka.ms/vs/17/release/vc_redist.x64.exe)
- Check Windows audio drivers
- Try different audio formats
- Restart Windows Audio service

**"Device not available" error:**

- Close other audio applications
- Check default audio device in Windows
- Try running as Administrator (once)

### File Issues

**"Invalid main_data offset" error:**

```
Error playing track 'Song Name': mpa: invalid main_data offset
This file may be corrupted. Try re-encoding or replacing it.
```

**Solutions:**

- Re-encode with modern tools (VLC, Audacity, FFmpeg)
- Download from reputable sources
- Check file integrity

### Performance Issues

**Slow first scan:**

- Normal for large libraries (builds cache for future speed)
- Monitor console output for progress
- Close other applications during first scan

**High memory usage:**

- Expected with very large libraries (50GB+)
- Memory usage scales with library size
- Consider splitting huge libraries

## 🏗️ Building from Source

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/W5DEV/music-shuffler.git
cd music-shuffler
```

### Local Build

```bash
# Build for current platform
cargo build --release

# Run locally
cargo run
```

### Windows Cross-Compilation (from macOS/Linux)

```bash
# One-time setup
./scripts/build-windows.sh
```

### GitHub Actions

- Automatic builds on every push
- Windows executables available in Actions artifacts
- Release builds triggered by tags

## 🤝 Contributing

We welcome contributions! Here's how to help:

1. **🐛 Bug Reports** - Open an issue with details
2. **💡 Feature Requests** - Suggest improvements
3. **🔧 Code Contributions** - Submit pull requests
4. **📖 Documentation** - Improve guides and docs
5. **🧪 Testing** - Test on different systems

### Development Setup

```bash
git clone https://github.com/W5DEV/music-shuffler.git
cd music-shuffler
cargo run
```

## 📊 Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   GUI (eGUI)    │    │  Audio (Rodio)  │    │ Files (WalkDir) │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ • Playlist UI   │    │ • MP3/FLAC/OGG  │    │ • Parallel Scan │
│ • Progress Bar  │    │ • Real-time     │    │ • Smart Filter  │
│ • Controls      │    │ • Low Latency   │    │ • Recursive     │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │ Metadata Cache  │
                    ├─────────────────┤
                    │ • JSON Storage  │
                    │ • Change Track  │
                    │ • Fast Lookup   │
                    └─────────────────┘
```

## 🎖️ Why Music Shuffler?

### vs. Other Players

- **Winamp/VLC:** More features, but complex setup
- **Spotify:** Streaming, but requires internet/subscription
- **Windows Media Player:** Built-in, but slow and dated
- **Music Shuffler:** Simple, fast, offline, just works

### Perfect For:

- 🏠 **Local Music Libraries** - Your own MP3/FLAC collection
- 🎲 **Discovery** - Rediscover forgotten songs in huge libraries
- ⚡ **Speed** - When you want music NOW, not after setup
- 🔒 **Privacy** - No tracking, no internet required
- 💻 **Low Resource** - Minimal impact on system performance

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Rust Community** - For amazing crates and ecosystem
- **eGUI** - For the beautiful, immediate-mode GUI framework
- **Rodio** - For reliable cross-platform audio
- **Contributors** - Everyone who helped improve this project

---

**Made with ❤️ and Rust**

_Music Shuffler: Because sometimes you just want to press play and discover great music._ 🎵
