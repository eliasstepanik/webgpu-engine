@echo off
echo Running game with FULL audio tracing enabled...
echo.
echo This will show ALL audio system activity (very verbose).
echo Look for messages like:
echo   - "Loading sound: ..."
echo   - "Playing sound: ..."
echo   - "Audio source playing"
echo.
set RUST_LOG=engine::audio=trace,symphonia=error
cargo run --features editor,audio -- %*