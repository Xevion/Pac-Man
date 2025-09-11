# Roadmap

A comprehensive list of features needed to complete the Pac-Man emulation, organized by priority and implementation complexity.

## Core Game Features

### Ghost AI & Behavior

- [x] Core Ghost System Architecture
  - [x] Ghost entity types (Blinky, Pinky, Inky, Clyde)
  - [x] Ghost state management (Normal, Frightened, Eyes)
  - [x] Ghost movement and pathfinding systems
- [ ] Authentic Ghost AI Personalities
  - [ ] Blinky (Red): Direct chase behavior
  - [ ] Pinky (Pink): Target 4 tiles ahead of Pac-Man
  - [ ] Inky (Cyan): Complex behavior based on Blinky's position
  - [ ] Clyde (Orange): Chase when far, flee when close
- [x] Mode Switching System
  - [ ] Scatter/Chase pattern with proper timing
  - [x] Frightened mode transitions
  - [ ] Ghost house entry/exit mechanics
- [x] Ghost House Behavior
  - [x] Proper spawning sequence
  - [ ] Exit timing and patterns
  - [ ] House-specific movement rules

### Fruit Bonus System

- [x] Fruit Spawning Mechanics
  - [x] Spawn at pellet counts 70 and 170
  - [x] Fruit display in bottom-right corner
  - [x] Fruit collection and scoring
  - [x] Bonus point display system

### Level Progression

- [ ] Multiple Levels
  - [ ] Level completion detection
  - [ ] Progressive difficulty scaling
  - [ ] Ghost speed increases per level
  - [ ] Power pellet duration decreases
- [ ] Intermission Screens
  - [ ] Between-level cutscenes
  - [ ] Proper graphics and timing

### Audio System Completion

- [x] Core Audio Infrastructure
  - [x] Audio event system
  - [x] Sound effect playback
  - [x] Audio muting controls
- [ ] Background Music
  - [x] Intro jingle
  - [ ] Continuous gameplay music
  - [ ] Escalating siren based on remaining pellets
  - [ ] Power pellet mode music
  - [ ] Intermission music
- [x] Sound Effects
  - [x] Pellet eating sounds
  - [x] Fruit collection sounds
  - [x] Ghost eaten sounds
  - [x] Pac-Man Death
  - [ ] Ghost movement sounds
  - [ ] Level completion fanfare

### Game Mechanics

- [ ] Bonus Lives
  - [ ] Extra life at 10,000 points
  - [x] Life counter display
- [ ] High Score System
  - [ ] High score tracking
  - [x] High score display
  - [ ] Score persistence

## Secondary Features (Medium Priority)

### Game Polish

- [x] Core Input System
  - [x] Keyboard controls
  - [x] Direction buffering for responsive controls
  - [x] Touch controls for mobile
- [x] Pause System
  - [x] Pause/unpause functionality
  - [ ] Pause menu with options
- [ ] Input System
  - [ ] Input remapping
  - [ ] Multiple input methods

## Advanced Features (Lower Priority)

### Difficulty Options

- [ ] Easy/Normal/Hard modes
- [ ] Customizable ghost speeds

### Data Persistence

- [ ] High Score Persistence
  - [ ] Save high scores to file
  - [ ] High score table display
- [ ] Settings Storage
  - [ ] Save user preferences
  - [ ] Audio/visual settings
- [ ] Statistics Tracking
  - [ ] Game statistics
  - [ ] Achievement system

### Debug & Development Tools

- [x] Performance details
- [x] Core Debug Infrastructure
  - [x] Debug mode toggle
  - [x] Comprehensive game event logging
  - [x] Performance profiling tools
- [ ] Game State Visualization
  - [ ] Ghost AI state display
  - [ ] Pathfinding visualization
  - [ ] Collision detection display
- [ ] Game Speed Controls
  - [ ] Variable game speed for testing
  - [ ] Frame-by-frame stepping

## Customization & Extensions

### Visual Customization

- [x] Core Rendering System
  - [x] Sprite-based rendering
  - [x] Layered rendering system
  - [x] Animation system
  - [x] HUD rendering
- [ ] Display Options
  - [x] Fullscreen support
  - [x] Window resizing
    - [ ] Pause while resizing (SDL2 limitation mitigation)
  - [ ] Multiple resolution support

### Gameplay Extensions

- [ ] Advanced Ghost AI
  - [ ] Support for >4 ghosts
  - [ ] Custom ghost behaviors
- [ ] Level Generation
  - [ ] Custom level creation
  - [ ] Multi-map tunneling
  - [ ] Level editor

## Online Features (Future)

### Scoreboard System

- [ ] Backend Infrastructure
  - [ ] Axum server with database
  - [ ] OAuth2 authentication
  - [ ] GitHub/Discord/Google auth
- [ ] Profile Features
  - [ ] Optional avatars (8-bit aesthetic)
  - [ ] Custom names (3-14 chars, filtered)
- [ ] Client Implementation
  - [ ] Zero-config client
  - [ ] Default API endpoint
  - [ ] Manual override available
