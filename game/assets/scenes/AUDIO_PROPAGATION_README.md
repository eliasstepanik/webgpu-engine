# Audio Propagation Demo

This demo showcases how sound propagates differently in various room types and configurations.

## Scene Overview

The scene contains three different acoustic environments:

1. **Small Room** (Left)
   - High absorption walls (0.4)
   - Low scattering (0.2)
   - Sound decays quickly
   - Simulates a carpeted bedroom or office

2. **Large Hall** (Right)
   - Low absorption walls (0.1)
   - High scattering (0.5)
   - Sound echoes and reverberates
   - Simulates a gymnasium or auditorium

3. **Narrow Hallway** (Center)
   - Medium absorption (0.2)
   - High scattering (0.6)
   - Sound travels down the corridor
   - Simulates a typical building hallway

## Audio Properties

Each room has a different sound source with unique properties:

- **Small Room**: Low pitch (0.7), high rolloff (3.0) - intimate sound
- **Large Hall**: High pitch (1.3), low rolloff (0.5) - spacious sound
- **Hallway**: Normal pitch (1.0), medium rolloff (2.5) - directional sound

## Controls

The scene includes an interactive controller script:

- **1/2/3**: Toggle individual room sounds
- **4**: Toggle all sounds on/off
- **Q/A**: Increase/Decrease volume
- **W/S**: Increase/Decrease pitch
- **Arrow Keys**: Move sound sources
- **WASD + Mouse**: Fly camera to explore

## Audio Material Properties

- **Absorption**: How much sound energy is absorbed (0.0 = reflective, 1.0 = absorptive)
- **Scattering**: How diffuse the reflections are (0.0 = specular, 1.0 = diffuse)
- **Transmission**: How much sound passes through (0.0 = solid, 1.0 = transparent)

## Loading the Demo

```bash
# With audio enabled
SCENE=audio_rooms_demo cargo run --features editor,audio

# Or use the batch file on Windows
run_with_audio.bat
```

Then set the SCENE environment variable:
```
set SCENE=audio_rooms_demo
```

## What to Listen For

1. **Distance Attenuation**: Move closer/farther from sound sources
2. **Occlusion**: Position walls between you and the sound
3. **Room Acoustics**: Compare how sounds behave in each room
4. **Directional Audio**: Sounds should come from the correct direction
5. **Environmental Effects**: Each room has distinct acoustic properties