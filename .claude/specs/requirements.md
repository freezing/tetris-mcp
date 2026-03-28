# Requirements: Claude Plays Tetris via MCP

## Introduction

Build a Tetris game engine in Rust exposed as an MCP (Model Context Protocol) server, enabling Claude to play Tetris by calling structured tools. The system records full game history (board states, moves, Claude's reasoning) and provides a viewer for replaying games — both as continuous playback for content creation and step-by-step for debugging. Lives in `tetris-mcp/` in the monorepo.

## Requirements

### Requirement 1: Tetris Engine
**User Story:** As a developer, I want a correct Tetris game engine as a Rust library crate, so that it can be used by the MCP server or any other consumer.

**Acceptance Criteria:**
1. WHEN the engine is initialized THEN the system SHALL create a standard 10-wide by 20-tall board with an empty grid.
2. WHEN a new piece is needed THEN the system SHALL spawn one of the 7 standard tetrominoes (I, O, T, S, Z, J, L) using a bag randomizer (7-bag system).
3. WHEN a piece is rotated THEN the system SHALL use the Super Rotation System (SRS) with wall kicks.
4. WHEN a piece is moved or rotated THEN the system SHALL detect collisions with the board boundaries and locked pieces.
5. WHEN a piece can no longer move down THEN the system SHALL lock it into the board.
6. WHEN one or more rows are completely filled THEN the system SHALL clear those rows and shift rows above down.
7. WHEN lines are cleared THEN the system SHALL update the score using standard Tetris scoring (single, double, triple, tetris).
8. WHEN a new piece spawns and immediately collides THEN the system SHALL declare game over.
9. WHEN any move is made THEN the system SHALL support: move left, move right, soft drop (move down), hard drop (instant drop to bottom), rotate clockwise, rotate counter-clockwise.
10. WHEN the game state is queried THEN the system SHALL provide: current board grid, current piece (type, position, orientation), next piece, score, lines cleared, game-over status.

### Requirement 2: MCP Server
**User Story:** As Claude (MCP client), I want to interact with the Tetris engine through structured MCP tools over stdio, so that I can play the game by calling tools each turn.

**Acceptance Criteria:**
1. WHEN the MCP server starts THEN the system SHALL communicate via JSON-RPC over stdio using the MCP protocol.
2. WHEN Claude calls `get_board_state` THEN the system SHALL return the current board grid as a 2D array with piece types indicated.
3. WHEN Claude calls `get_current_piece` THEN the system SHALL return the piece type, position, and orientation.
4. WHEN Claude calls `get_next_piece` THEN the system SHALL return the upcoming piece type.
5. WHEN Claude calls `move_piece` with a direction (left, right, down) THEN the system SHALL execute the move and return the updated state.
6. WHEN Claude calls `rotate` with a direction (cw, ccw) THEN the system SHALL execute the rotation and return the updated state.
7. WHEN Claude calls `hard_drop` THEN the system SHALL drop the piece to the bottom, lock it, and return the updated state.
8. WHEN Claude calls `get_score` THEN the system SHALL return current score, lines cleared, and level.
9. IF a move is invalid (e.g., moving into a wall) THEN the system SHALL return an error message describing why.
10. IF the game is over THEN tool calls that attempt moves SHALL return an error indicating game over.

### Requirement 2b: Instructions Endpoint
**User Story:** As Claude (MCP client), I want an instructions endpoint that explains the game rules, available tools, and the goal, so that I can play effectively without prior Tetris knowledge.

**Acceptance Criteria:**
1. WHEN Claude calls `get_instructions` THEN the system SHALL return a structured description containing: the goal of Tetris (clear lines by filling complete rows), how pieces work (7 types, rotation, movement), what each tool does with its parameters, scoring rules, and what causes game over.
2. WHEN instructions are returned THEN the system SHALL include a suggested play loop (e.g., "call get_board_state, decide placement, use move/rotate/hard_drop, repeat").
3. WHEN instructions are returned THEN the system SHALL document all available tools with their parameters and return types.

### Requirement 3: Game History & Replay
**User Story:** As a researcher, I want every game to be fully recorded with board states, moves, and Claude's reasoning, so that I can replay games, compare runs across different strategies, and iterate on Claude's skill.

**Acceptance Criteria:**
1. WHEN a game starts THEN the system SHALL begin recording a history log.
2. WHEN any move is made THEN the system SHALL record: move type, timestamp, board state before the move, board state after the move, current piece, score, and lines cleared.
3. WHEN Claude provides reasoning (via tool call context) THEN the system SHALL store that reasoning alongside the corresponding move.
4. WHEN the game ends THEN the system SHALL write the complete game to its own JSON file in a `games/` directory, named with a timestamp or unique ID (e.g., `games/2026-03-28T14-30-00.json`).
5. WHEN a game file is written THEN it SHALL include metadata: start time, end time, final score, total moves, lines cleared, pieces placed, game-over reason, and the system prompt / strategy label used for that run.
6. WHEN a history file is loaded THEN the system SHALL support reconstructing any game state at any point in the history.
7. WHEN the viewer opens without a specific file THEN it SHALL list all games in the `games/` directory sorted by date, showing metadata (score, lines, moves, strategy label) for quick comparison.

### Requirement 3b: Multi-Game Analysis
**User Story:** As a researcher, I want to compare performance across games with different strategies/prompts, so that I can measure whether skills or prompt changes improve Claude's play.

**Acceptance Criteria:**
1. WHEN games are recorded THEN each game's metadata SHALL include a configurable `strategy` label (e.g., "baseline", "think-step-by-step", "custom-skill-v1") passed at server startup.
2. WHEN the viewer lists games THEN it SHALL allow filtering/grouping by strategy label.
3. WHEN multiple games exist THEN the viewer SHALL show summary stats per strategy: average score, average lines cleared, average survival time, number of games played.

### Requirement 4: Game Viewer
**User Story:** As a content creator, I want to visually replay recorded games, so that I can create content showing Claude playing Tetris and debug its strategy.

**Acceptance Criteria:**
1. WHEN a replay file is loaded THEN the viewer SHALL render the Tetris board with colored pieces.
2. WHEN in continuous playback mode THEN the viewer SHALL animate moves at a configurable speed.
3. WHEN in step-by-step mode THEN the viewer SHALL advance one move per user action (keyboard/click).
4. WHEN stepping through moves THEN the viewer SHALL display Claude's reasoning for each move alongside the board.
5. WHEN replaying THEN the viewer SHALL show: current score, lines cleared, move number, and total moves.
6. IF playback is paused THEN the viewer SHALL allow scrubbing forward/backward to any move.
