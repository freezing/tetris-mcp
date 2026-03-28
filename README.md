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

**Game History** — Every game saved as a JSON file with full move-by-move state, Claude's reasoning, and metadata (score, strategy label). Supports comparing runs across different prompts/skills.

**Web Viewer** — HTML/JS replay viewer for watching games back. Continuous playback for content creation, step-by-step for debugging Claude's decisions.

## Project Structure

```
tetris-mcp/
├── engine/          # Tetris game engine (library crate)
├── mcp-server/      # MCP server binary
├── viewer/          # HTML/JS replay viewer
└── games/           # Recorded game history (JSON)
```

## Status

Work in progress. The engine is functional. MCP server and viewer coming next.

---

The majority of this project was vibe-coded with [Claude Code](https://claude.ai/code).
