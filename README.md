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

## Viewer

The web viewer replays recorded games with animated hard drops, step-by-step controls, and scrubbing.

### Build

```bash
wasm-pack build engine-wasm --target web --out-dir ../viewer/pkg
```

### Run

```bash
cd viewer
python3 -m http.server 8080
```

Open `http://localhost:8080` and drag a game JSON file onto the page. Game files are saved to `games/` after each MCP session.

### Controls

| Key | Action |
|-----|--------|
| Space | Play / Pause |
| Right arrow | Step forward |
| Left arrow | Step back |
| Home | Jump to start |
| End | Jump to end |
| Drag slider | Scrub timeline |
| Speed dropdown | 0.5x to 10x |

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
      "args": []
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
```

### Testing with MCP Inspector

You can test the server interactively without Claude using the official MCP inspector:

```bash
npx @modelcontextprotocol/inspector tetris-mcp-server -- --seed 42
```

This opens a web UI where you can browse tools, call them manually, and see the responses.

Games are saved to `games/` as JSON files. Load them in the viewer to replay and compare.

## How Well Does Claude Play?

Short answer: not great. Claude understands Tetris strategy but can't execute it.

### What Claude tries

- Keep the board flat by alternating pieces left and right
- Reserve column 9 for vertical I-piece Tetrises (4-line clears)
- Fill gaps in lower rows before adding height
- Rotate pieces to fit tight spaces (L-West for narrow columns, T-South to cap gaps)

### What actually happens

Claude can't reliably predict where pieces will land. The core issue is **spatial simulation** — given a piece shape and a board state, computing which cells the piece will occupy when it locks. This is trivial for a game engine but difficult when your only representation is ASCII text and coordinate lists.

Specific failure modes:
- **S/Z pieces** create 2-row height staggers that cascade into uneven surfaces
- **Landing height errors** — Claude plans to fill row 19 but the piece lands at row 16 because an adjacent column is taller than expected, creating permanent holes
- **Inaccessible holes** accumulate (3-5 per game), making line clears impossible

### Benchmark results

Tested across 3 configurations: high-effort Opus, low-effort Sonnet, and two modes per config (speed = 2s per move, careful = no time limit).

| Configuration | Speed Avg | Careful Avg | Lines Cleared |
|---|---|---|---|
| High effort (Opus) | 480 | 502 | 0 |
| Low effort (Sonnet) | 418 | 399 | 0 |

Game files are in [`games/`](games/) — load them in the viewer to replay.

<details>
<summary>Individual game scores</summary>

**High Effort (Opus)**

| # | Mode | Score | Game File |
|---|---|---|---|
| 1 | speed | 468 | [`0337da12`](games/0337da12-be01-42c9-a1e8-ebf5bd4dc270.json) |
| 2 | speed | 484 | [`a269c166`](games/a269c166-21e1-43fa-9b6e-dde09b317eb3.json) |
| 3 | speed | 488 | [`e0c29c49`](games/e0c29c49-67f8-4246-94e3-234c8b57fecc.json) |
| 4 | careful | 496 | [`93f32d94`](games/93f32d94-87a0-44cc-9a1d-5eeb425e3b32.json) |
| 5 | careful | 530 | [`245cbdcf`](games/245cbdcf-e076-4072-9d04-f86c337089bd.json) |
| 6 | careful | 480 | [`a4f89d7c`](games/a4f89d7c-40b6-4d86-8f25-6db75b17fd00.json) |

**Low Effort (Sonnet)**

| # | Mode | Score | Game File |
|---|---|---|---|
| 1 | speed | 478 | [`974267f6`](games/974267f6-1445-44e5-af1f-c3fb7bb69c9c.json) |
| 2 | speed | 402 | [`d000e4ae`](games/d000e4ae-f2de-4b83-93b4-b5711aad1db6.json) |
| 3 | speed | 374 | [`aee4d5e5`](games/aee4d5e5-4782-41cc-b15d-b328a3b072e8.json) |
| 4 | careful | 418 | [`f37a8222`](games/f37a8222-c344-4eef-b9cf-f7c4e8410ba4.json) |
| 5 | careful | 386 | [`eff11e53`](games/eff11e53-5f89-48e9-9bc4-1edc44da2f45.json) |
| 6 | careful | 392 | [`6004e39e`](games/6004e39e-3d30-499c-a438-2ae3551bed0e.json) |

</details>

- **Model capability > thinking time.** Opus averages ~490; Sonnet averages ~408. The 17% gap between models is larger than the speed/careful gap within either.
- **Thinking time only helps with a strong model.** Opus careful beats speed by 5%. Sonnet careful is actually *worse* than speed — extra time doesn't help when reasoning capacity is limited.
- **Line clears are rare and lucky.** In 12 controlled games, zero lines cleared. One early outlier hit 1,700 with 5 line clears — likely noise from a favorable piece sequence rather than skill. It was never reproduced.
- **Score is almost entirely hard-drop points.** With no clears, score = 2 pts/row dropped. Games last 17-25 pieces before topping out (370-530 points).

## Status

- Engine: done
- WASM viewer: done
- MCP server: done
- Claude integration: ready

---

The majority of this project was vibe-coded with [Claude Code](https://claude.ai/code).
