@echo off
echo Running editor with debugging output...
set RUST_LOG=debug,wgpu_core=warn,wgpu_hal=warn
cargo run --features editor
pause