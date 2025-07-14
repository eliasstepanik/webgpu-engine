#!/bin/bash
echo "Starting focused debug test..."
echo "This will only show property changes and periodic status reports."
echo ""
echo "Instructions:"
echo "1. Load scene: game/assets/scenes/test_property_persistence.json"
echo "2. Select a rotating cube"
echo "3. Try to change the rotation_speed value"
echo "4. Watch for messages marked with 🚨 (property changes) or 💡 (inspector changes)"
echo ""
echo "Starting in 3 seconds..."
sleep 3

cargo run --features editor 2>&1 | grep -E "(🚨|💡|📊|PROPERTY|INSPECTOR|STATUS REPORT)" | grep -v "DEBUG PROPERTY INIT"