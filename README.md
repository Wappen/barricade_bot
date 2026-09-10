# TODO:

## API Design

**Distance & Pathfinding Queries**

- `fn shortest_path_len(&self, team: Team) -> Option<usize>`: Instantly returns the exact length of the optimal path to the victory row, which is the foundational metric for race evaluation.
- `fn all_shortest_paths(&self, team: Team) -> Vec<Vec<PCoord>>`: Retrieves every equally optimal path. This helps bots identify chokepoints or shared path segments where a single wall blocks multiple routes.
- `fn distance_to_goal(&self, coord: PCoord, team: Team) -> usize`: Evaluates potential future positions without running a full path search from scratch.

**Action Generation & Move Filtering**

- `fn legal_moves(&self, team: Team) -> Vec<PCoord>`: Yields all valid standard steps, jumps, and diagonal shifts for a specific player, streamlining move loops.
- `fn legal_barricades(&self) -> Vec<(BCoord, Orientation)>`: Generates every structurally valid wall placement that doesn't illegally trap either player.
- `fn evaluating_wall_impact(&self, coord: BCoord, orientation: Orientation) -> WallMetrics`: Simulates placing a wall and computes the exact delta it adds to the opponent's shortest path versus your own. Essential for defensive wall placement algorithms.

**Tactical & Graph Analysis**

- `fn find_chokepoints(&self, team: Team) -> Vec<PCoord>`: Identifies critical graph articulation points or narrow nodes where the enemy has zero alternative routing flexibility.
- `fn evaluates_race_advantage(&self) -> i32`: Calculates `enemy_distance - self_distance`. A positive value means your bot is winning the race; a negative value signals it's time to switch to aggressive wall blocking.
- `fn opponent_coord(&self, team: Team) -> PCoord`: Quick lookup helper for tracking enemy position relative to your own trajectory.
