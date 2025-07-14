#!/bin/bash

echo "Testing material updates in scripts..."
echo "=========================="

# Run with debug logging for scripting system
export RUST_LOG="warn,engine::scripting=debug,engine::scripting::commands=debug,engine::scripting::modules::world=trace"

# Run the game with the test scene
cargo run --bin game -- --scene test_color_pulse