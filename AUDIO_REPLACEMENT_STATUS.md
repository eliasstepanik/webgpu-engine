# Audio Library Replacement Status

## Summary
Successfully replaced Kira audio library with Rodio/CPAL for better audio device selection support.

## Changes Made

### 1. Dependencies
- Added `rodio = { version = "0.17", optional = true }` to engine/Cargo.toml
- Added `audio = ["rodio"]` feature flag

### 2. Core Implementation
- Replaced AudioEngine implementation in `engine/src/audio/engine.rs`
- AudioHandle now wraps `rodio::Sink` instead of Kira handles
- Maintained all existing public API methods

### 3. New Features
- `AudioEngine::enumerate_devices()` - Lists available audio output devices
- `AudioEngine::set_output_device()` - Switches to specific audio device
- `AudioEngine::get_current_device()` - Returns current device name

### 4. Editor Integration
- Updated settings dialog to show audio device dropdown
- Devices are enumerated dynamically using the new API
- Default device is clearly marked

### 5. Stub Implementation
- Updated `audio_stub.rs` to match new API surface
- Builds correctly without audio feature

## Testing Status

### ✅ Completed
- Code compiles without audio feature
- No clippy warnings
- All API compatibility maintained
- Editor UI integration complete

### ⚠️ Pending (Platform Specific)
- Linux: Requires ALSA dependencies (libasound2-dev, pkg-config)
- Windows: Should work out of the box
- Audio demo scene testing
- Device switching at runtime

## Known Limitations
1. Master volume control not implemented (Rodio limitation)
2. Fade in/out not implemented (would need custom implementation)
3. Device switching stops all currently playing sounds

## Next Steps
1. Test on Windows with actual audio devices
2. Test audio_demo.json scene functionality
3. Verify device switching works at runtime
4. Consider implementing master volume tracking

## Build Instructions

### Without Audio
```bash
cargo build
```

### With Audio
```bash
# Linux: First install ALSA dependencies
sudo apt-get install libasound2-dev pkg-config

# Then build
cargo build --features audio
```

### Run with Audio
```bash
cargo run --features audio
```