#!/bin/bash
echo "Testing scene loading and property persistence..."
echo ""
echo "Watch for these key messages:"
echo "📦 SCENE LOAD - shows what values are loaded from the scene file"
echo "🚨 PROPERTY CHANGED - shows when properties unexpectedly change"
echo "💡 INSPECTOR CHANGED - shows when you edit in the inspector"
echo "🔄 Entity not in started_entities - shows when scripts restart"
echo ""
echo "Starting in 3 seconds..."
sleep 3

cargo run --features editor 2>&1 | grep -E "(📦|🚨|💡|🔄|SCENE LOAD|PROPERTY CHANGED|INSPECTOR CHANGED|started_entities)" | grep -v "focused_debug"