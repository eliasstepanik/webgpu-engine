@echo off
echo Running game with audio debugging enabled...
echo.
echo This will show detailed audio system logs to help diagnose playback issues.
echo.
set RUST_LOG=engine::audio=debug,symphonia=error
cargo run --features editor,audio --bin game -- %*