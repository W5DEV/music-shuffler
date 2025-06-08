# Windows Deployment Guide

## 🚀 Quick Start Options

### Option 1: GitHub Actions (Recommended) ⭐

**Automatic builds on every commit**

1. Push code to GitHub
2. GitHub Actions automatically builds Windows .exe
3. Download from Actions tab or Releases

**Pros:** Automatic, tested, reliable  
**Cons:** Requires GitHub repo

### Option 2: Local Cross-Compilation

**Build Windows .exe from macOS/Linux**

```bash
# One-time setup
brew install mingw-w64  # macOS
# sudo apt install mingw-w64 gcc-mingw-w64  # Linux

# Build
./scripts/build-windows.sh
```

**Pros:** Local control, no dependencies  
**Cons:** Requires setup, potential compatibility issues

### Option 3: Native Windows Build

**Build directly on Windows machine**

```powershell
# Install Rust if not already
winget install Rustlang.Rust.GNU

# Build
cargo build --release
```

**Pros:** Maximum compatibility  
**Cons:** Requires Windows machine

## 📦 Distribution Options

### Simple ZIP Distribution

```
music-shuffler-windows.zip
├── music-shuffler.exe
└── README.txt
```

**Best for:** Personal use, technical users  
**Deployment:** Upload to GitHub Releases, file sharing

### Professional Installer (Inno Setup)

```
music-shuffler-setup.exe
```

**Features:**

- ✅ Proper Windows installer experience
- ✅ Start Menu shortcuts
- ✅ Desktop icon (optional)
- ✅ Uninstaller
- ✅ Auto-detects missing dependencies
- ✅ Registry integration

**Best for:** Wide distribution, non-technical users

## 🔧 Build Commands Summary

### GitHub Actions (Automatic)

```yaml
# Triggered on push to main branch
# Downloads from GitHub Actions artifacts
```

### Local macOS/Linux

```bash
chmod +x scripts/build-windows.sh
./scripts/build-windows.sh
```

### Local Windows

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

### Cross-compile from macOS

```bash
rustup target add x86_64-pc-windows-gnu
brew install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```

## 📊 Expected File Sizes

| Component          | Size     | Description                              |
| ------------------ | -------- | ---------------------------------------- |
| music-shuffler.exe | ~15-25MB | Main executable (includes GUI framework) |
| README.txt         | ~2KB     | User documentation                       |
| Total ZIP          | ~15-25MB | Complete package                         |
| Installer          | ~16-26MB | Inno Setup installer                     |

## 🎯 Testing Strategy

### Minimum Test Matrix

- ✅ **Fresh Windows 10/11** (no dev tools)
- ✅ **Windows with basic codecs** (Windows Media Player installed)
- ✅ **Different audio formats** (MP3, FLAC, OGG)
- ✅ **Large music library** (10GB+)

### Test Checklist

- [ ] App launches without errors
- [ ] Directory selection works
- [ ] File scanning completes
- [ ] Playlist generation works
- [ ] Audio playback functions
- [ ] Progress bar updates
- [ ] Previous/Next buttons work
- [ ] App remembers directory

## 🐛 Common Issues & Solutions

### "VCRUNTIME140.dll not found"

**Solution:** Install Visual C++ Redistributables

```
https://aka.ms/vs/17/release/vc_redist.x64.exe
```

### "No audio output"

**Solutions:**

1. Check Windows audio drivers
2. Try different audio formats
3. Close other audio applications
4. Restart Windows Audio service

### "App won't start"

**Solutions:**

1. Run as Administrator (once)
2. Check Windows Defender/Antivirus
3. Install .NET Framework (if needed)
4. Verify Windows version compatibility

## 🎯 Recommended Deployment Flow

### For Personal Use:

1. Use GitHub Actions
2. Download ZIP from releases
3. Extract and run

### For Distribution:

1. Build with GitHub Actions
2. Test on clean Windows VM
3. Create Inno Setup installer
4. Sign executable (optional)
5. Distribute via website/GitHub

## 🔐 Code Signing (Optional)

For professional distribution:

```bash
# Get code signing certificate
# Sign the executable
signtool sign /f certificate.p12 /p password /t http://timestamp.digicert.com music-shuffler.exe
```

**Benefits:**

- ✅ Removes Windows security warnings
- ✅ Professional appearance
- ✅ User trust

**Cost:** ~$200-400/year for certificate

## 📈 Performance Expectations

### First Launch:

- App start: < 2 seconds
- Directory scan (10GB): 30-60 seconds
- Cached subsequent scans: < 5 seconds

### Runtime:

- Memory usage: 50-100MB
- CPU usage: < 5% when playing
- Disk usage: ~25MB + cache files

The app is optimized for Windows deployment and should work out-of-the-box on most modern Windows systems! 🎵
