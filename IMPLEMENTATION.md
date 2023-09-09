# Implementation

A document detailing the implementation the project from rendering, to game logic, to build systems.

## Rendering

1. Map
    - May require procedural text generation later on (cacheable?)
2. Pacman
3. Ghosts
    - Requires colors
4. Items
5. Interface
    - Requires fonts

## Grid System

1. How does the grid system work?

The grid is 28 x 36 (although, the map texture is 28 x 37), and each cell is 24x24 (pixels).
Many of the walls in the map texture only occupy a portion of the cell, so some items are able to render across multiple cells.
24x24 assets include pellets, the energizer, and the map itself ()

2. What constraints must be enforced on Ghosts and PacMan?

3. How do movement transitions work?

All entities store a precise position, and a direction. This position is only used for animation, rendering, and collision purposes. Otherwise, a separate 'cell position' (which is 24 times less precise, owing to the fact that it is based on the entity's position within the grid).

When an entity is transitioning between cells, movement directions are acknowledged, but won't take effect until the next cell has been entered completely.

4. Between transitions, how does collision detection work?

It appears the original implementation used cell-level detection.
I worry this may be prone to division errors. Make sure to use rounding (50% >=).