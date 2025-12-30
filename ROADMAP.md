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
- [ ] Mode Switching System
  - [ ] Scatter/Chase pattern with proper timing
  - [x] Frightened mode transitions
  - [ ] Ghost house entry/exit mechanics
- [ ] Ghost House Behavior
  - [x] Basic spawning in ghost house
  - [ ] Proper exit timing sequence
  - [ ] Dot counter for ghost release
  - [ ] Global dot counter reset
  - [ ] House-specific pathfinding

### Fruit Bonus System

- [x] Fruit Spawning Mechanics
  - [x] Spawn at pellet counts 5 and 170 (TODO: verify if should be 70 and 170 for arcade accuracy)
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
  - [x] Keyboard controls (WASD + Arrow keys)
  - [x] Direction buffering for responsive controls
  - [x] Touch controls for mobile
  - [x] Mouse controls (mouse-as-touch for desktop testing)
- [x] Pause System
  - [x] Pause/unpause functionality (Escape key)
  - [x] Visual pause overlay with semi-transparent background
  - [ ] Interactive pause menu with options
- [ ] Advanced Input System
  - [ ] Input remapping/key rebinding
  - [ ] Customizable control schemes

## Web Platform Features

### Browser Compatibility

- [x] Core Web Support
  - [x] WASM compilation and deployment
  - [x] Emscripten platform integration
  - [x] Browser autoplay policy compliance (click-to-start)
  - [x] WASM loading states and smooth transitions
- [x] Mobile Web Support
  - [x] Touch controls for mobile browsers
  - [x] Responsive touch input handling
  - [x] Mobile-optimized UI

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
  - [x] Debug mode toggle (Space key)
  - [x] Comprehensive game event logging
  - [x] Per-system performance profiling with timing breakdown
  - [x] FPS and frame timing display
  - [x] Single-tick stepping (T key for frame-by-frame)
- [x] Maze Visualization
  - [x] Navigation graph rendering (red lines)
  - [x] Node visualization (blue dots)
  - [x] Node hover inspection with ID display
  - [x] Collision hitbox display (green boxes)
- [ ] Game State Visualization
  - [ ] Ghost AI state display (target tiles, modes)
  - [ ] Ghost pathfinding visualization (current path)
  - [ ] Ghost personality indicators
- [ ] Game Speed Controls
  - [ ] Variable game speed for testing

## Customization & Extensions

### Visual Customization

- [x] Core Rendering System
  - [x] Sprite-based rendering
  - [x] Layered rendering system
  - [x] Animation system
  - [x] HUD rendering
- [ ] Display Options
  - [x] Fullscreen support (Desktop only - F key toggle)
  - [ ] Fullscreen support (Web - F key + pinch-to-zoom gestures)
  - [x] Window resizing with aspect ratio preservation
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

- [x] Backend Infrastructure
  - [x] Axum server with PostgreSQL database
  - [x] Optional OAuth2 authentication (GitHub/Discord)
  - [x] Session management with cookies
  - [x] Health check and API endpoints
- [ ] Profile Features
  - [ ] Optional avatars (8-bit aesthetic)
  - [ ] Custom names (3-14 chars, filtered)
  - [ ] User profile pages
- [ ] Leaderboard Features
  - [ ] Global high score board
  - [ ] Personal best tracking
  - [ ] Score submission from game client
- [ ] Client Implementation
  - [ ] Zero-config client with default endpoint
  - [ ] Score upload integration
  - [ ] Manual API endpoint override
