#!/bin/bash
# Test script for audio system

echo "Testing audio system build..."

# Check if we're on Windows or Linux
if [[ "$OSTYPE" == "linux-gnu"* ]] || [[ "$OSTYPE" == "linux" ]]; then
    echo "Linux detected. Checking for ALSA dependencies..."
    if ! command -v pkg-config &> /dev/null; then
        echo "ERROR: pkg-config not found. Please install: sudo apt-get install pkg-config"
        exit 1
    fi
    if ! pkg-config --exists alsa; then
        echo "ERROR: ALSA not found. Please install: sudo apt-get install libasound2-dev"
        exit 1
    fi
fi

echo "Building with audio feature..."
cargo build --features audio

if [ $? -eq 0 ]; then
    echo "Audio build successful!"
    echo ""
    echo "To run with audio:"
    echo "  cargo run --features audio"
    echo ""
    echo "Then load the audio_demo.json scene in the editor."
else
    echo "Audio build failed. See errors above."
    exit 1
fi