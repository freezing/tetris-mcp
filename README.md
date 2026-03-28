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
├── mcp-server/      # MCP server binary
├── viewer/          # HTML/JS replay viewer
│   ├── index.html   # Viewer app
│   ├── pkg/         # WASM build output (generated)
│   └── examples/    # Sample game files
└── games/           # Recorded game history (gitignored)
```

## Playing with Claude

### Install the MCP server

```bash
cargo install --path mcp-server
```

This installs `tetris-mcp-server` to `~/.cargo/bin/`.

### Add to Claude Code

Add to your project's `.claude/settings.json`:

```json
{
  "mcpServers": {
    "tetris": {
      "command": "tetris-mcp-server",
      "args": ["--strategy", "baseline"]
    }
  }
}
```

Then start a new Claude Code session. Claude will have access to Tetris tools. Tell it to play:

> Call get_instructions to learn the rules, then play a game of Tetris.

### CLI options

```
tetris-mcp-server [OPTIONS]

Options:
  --width <WIDTH>          Board width [default: 10]
  --height <HEIGHT>        Board height [default: 20]
  --seed <SEED>            RNG seed (random if not set)
  --games-dir <DIR>        Where to save game history [default: games]
  --strategy <LABEL>       Strategy label for comparing runs [default: default]
```

### Testing with MCP Inspector

You can test the server interactively without Claude using the official MCP inspector:

```bash
npx @modelcontextprotocol/inspector tetris-mcp-server -- --seed 42
```

This opens a web UI where you can browse tools, call them manually, and see the responses.

### Comparing strategies

Run multiple games with different prompts and compare:

```bash
# Game 1: baseline
tetris-mcp-server --strategy "baseline"

# Game 2: with coaching
tetris-mcp-server --strategy "think-step-by-step"
```

Games are saved to `games/` as JSON files. Load them in the viewer to replay and compare.

## Status

- Engine: done
- WASM viewer: done
- MCP server: done
- Claude integration: ready

---

The majority of this project was vibe-coded with [Claude Code](https://claude.ai/code).
