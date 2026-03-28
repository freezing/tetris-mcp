# Tetris MCP

A Tetris engine exposed as an [MCP](https://modelcontextprotocol.io/) (Model Context Protocol) server, letting Claude play Tetris by calling structured tools. Built in Rust.

## What is this?

Claude connects to the Tetris MCP server and plays the game by calling tools like `get_board_state`, `move_piece`, `rotate`, and `hard_drop`. Every game is recorded with full board states, moves, and Claude's reasoning — so you can replay games, compare strategies, and see how an LLM handles spatial reasoning in real time.

## Why?

Just for fun!

I thought it would be interesting to get an LLM to play a game where it's unlikely to be good at. I'm curious how it would do.

## Architecture

```
┌──────────────┐     MCP (stdio)     ┌───────────────┐
│   Claude     │ ◄─────────────────► │  MCP Server   │
│  (MCP client)│                     │  (Rust bin)   │
└──────────────┘                     └─────┬─────────┘
                                           │
                                    ┌──────▼─────────┐
                                    │  Tetris Engine │
                                    │  (Rust lib)    │
                                    └──────┬─────────┘
                                           │
                                    ┌──────▼─────────┐
                                    │  Game History  │
                                    │  (JSON files)  │
                                    └──────┬─────────┘
                                           │
                                    ┌──────▼─────────┐
                                    │  Web Viewer    │
                                    │  (HTML/JS)     │
                                    └────────────────┘
```

**Engine** — Rust library crate with standard Tetris rules: 7-bag randomizer, SRS rotation with wall kicks, line clearing, scoring. Deterministic via seeded RNG. The engine is turn-based with no gravity — pieces don't fall automatically. Claude explicitly moves and drops each piece, making the game a pure spatial reasoning test with no time pressure. This keeps things simple and fair for an LLM player. Gravity and move budgets can be added later as difficulty modes.

**MCP Server** — Rust binary that wraps the engine and exposes it as MCP tools over stdio. Claude calls tools to play.

**Game History** — Every game saved as a JSON file with delta-encoded moves (not full board snapshots). The grid uses sparse serialization — only occupied cells are stored. A typical 50-move game is ~20KB.

**Web Viewer** — HTML/JS replay viewer powered by the engine compiled to WASM. Loads game history files and replays them with animated hard drops, step-by-step controls, and scrubbing.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (for the viewer)

### Run the tests

```bash
cargo test -p engine
```

### Generate a sample game

```bash
cargo run --example sample_game > viewer/examples/sample_game.json
```

### Build the viewer

```bash
wasm-pack build engine-wasm --target web --out-dir ../viewer/pkg
```

### Launch the viewer

```bash
cd viewer
python3 -m http.server 8080
```

Open `http://localhost:8080` and drag a game JSON file onto the page.

**Viewer controls:**
- **Play/Pause** — spacebar or Play button
- **Step forward/back** — arrow keys or Prev/Next buttons
- **Jump to start/end** — Home/End keys
- **Scrub** — drag the progress slider
- **Speed** — dropdown (0.5x to 10x)

Hard drops animate the piece falling rather than teleporting.

## Project Structure

```
tetris-mcp/
├── engine/          # Tetris game engine (library crate)
├── engine-wasm/     # WASM wrapper for the viewer
├── mcp-server/      # MCP server binary (WIP)
├── viewer/          # HTML/JS replay viewer
│   ├── index.html   # Viewer app
│   ├── pkg/         # WASM build output (generated)
│   └── examples/    # Sample game files
└── games/           # Recorded game history (gitignored)
```

## Status

- Engine: done
- WASM viewer: done
- MCP server: not started
- Claude integration: not started

---

The majority of this project was vibe-coded with [Claude Code](https://claude.ai/code).
