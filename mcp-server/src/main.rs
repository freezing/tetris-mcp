use std::sync::Mutex;

use engine::{Direction, Game, MoveResult, RotateDirection};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "tetris-mcp", about = "Tetris MCP server — let Claude play Tetris")]
struct Args {
    /// Board width
    #[arg(long, default_value_t = 10)]
    width: usize,

    /// Board height
    #[arg(long, default_value_t = 20)]
    height: usize,

    /// RNG seed (random if not specified)
    #[arg(long)]
    seed: Option<u64>,

    /// Directory to save game history
    #[arg(long)]
    games_dir: Option<String>,

}

#[derive(Clone)]
pub struct TetrisServer {
    game: std::sync::Arc<Mutex<Game>>,
    games_dir: String,
    tool_router: ToolRouter<TetrisServer>,
}

// --- Tool parameter types ---

#[derive(Deserialize, schemars::JsonSchema)]
struct MoveRequest {
    /// Direction to move: "left", "right", or "down"
    direction: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct RotateRequest {
    /// Rotation direction: "cw" (clockwise) or "ccw" (counter-clockwise)
    direction: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct NewGameRequest {
    /// Optional tag label for tagging this game (e.g. "baseline", "aggressive")
    tag: Option<String>,
}

// --- Tools ---

#[tool_router]
impl TetrisServer {
    #[tool(description = "Get instructions on how to play Tetris. Call this first to understand the game rules, available tools, and the goal.")]
    fn get_instructions(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text(
            r#"# Tetris — How to Play

## Goal
Clear lines by filling complete horizontal rows with pieces. Each cleared line scores points. The game ends when a new piece can't spawn because the board is full.

## The Board
10 columns wide, 20 rows tall. Pieces spawn at the top center.

## Pieces
There are 7 piece types: I, O, T, S, Z, J, L. Each has a unique shape.
Pieces arrive in shuffled bags of all 7 — you'll see each piece once per bag.

## Available Tools

1. **get_board_state** — Returns the current board grid, showing locked pieces. Empty cells are null.
2. **get_current_piece** — Returns the active piece: its type, position (x, y), and rotation.
3. **get_next_piece** — Returns the type of the upcoming piece.
4. **move_piece** — Move the active piece. Pass direction: "left", "right", or "down". Moving down into an obstacle locks the piece.
5. **rotate** — Rotate the active piece. Pass direction: "cw" (clockwise) or "ccw" (counter-clockwise). Uses wall kicks if near edges.
6. **hard_drop** — Instantly drop the piece to the bottom and lock it. Scores 2 points per row dropped.
7. **get_score** — Returns current score, lines cleared, and level.

## Suggested Play Loop
1. Call get_board_state and get_current_piece to see the situation
2. Decide where to place the piece (consider get_next_piece for planning)
3. Use move_piece and rotate to position it
4. Call hard_drop to lock it in place
5. Repeat

## Scoring
- 1 line: 100 × level
- 2 lines: 300 × level
- 3 lines: 500 × level
- 4 lines (Tetris): 800 × level
- Hard drop: 2 points per row

## Tips
- Try to keep the board flat
- Leave a column open for I-pieces to score Tetrises (4-line clears)
- Avoid creating holes under pieces — they're hard to clear
"#,
        )]))
    }

    #[tool(description = "Get the current board state. Returns a grid of cells — each is either null (empty) or a piece type letter (I, O, T, S, Z, J, L) for locked pieces.")]
    fn get_board_state(&self) -> Result<CallToolResult, McpError> {
        let game = self.game.lock().unwrap();
        let board = &game.state().board;
        let mut rows = Vec::new();
        for y in 0..board.height() {
            let mut row = Vec::new();
            for x in 0..board.width() {
                match board.grid.get(x, y) {
                    Some(p) => row.push(format!("{p:?}")),
                    None => row.push(".".to_string()),
                }
            }
            rows.push(format!("{:>2} |{}|", y, row.join("")));
        }
        let header = format!("    {}", (0..board.width()).map(|x| x.to_string()).collect::<String>());
        Ok(CallToolResult::success(vec![Content::text(
            format!("{}\n{}", header, rows.join("\n")),
        )]))
    }

    #[tool(description = "Get the current active piece — its type, position (x, y), and rotation state.")]
    fn get_current_piece(&self) -> Result<CallToolResult, McpError> {
        let game = self.game.lock().unwrap();
        let piece = &game.state().current_piece;
        let cells: Vec<String> = piece.cells().iter().map(|(x, y)| format!("({x},{y})")).collect();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Type: {:?}\nPosition: ({}, {})\nRotation: {:?}\nCells: {}",
            piece.piece_type, piece.x, piece.y, piece.rotation, cells.join(", ")
        ))]))
    }

    #[tool(description = "Get the next piece type that will spawn after the current piece is placed.")]
    fn get_next_piece(&self) -> Result<CallToolResult, McpError> {
        let game = self.game.lock().unwrap();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "{:?}",
            game.state().next_piece
        ))]))
    }

    #[tool(description = "Move the active piece. Direction must be \"left\", \"right\", or \"down\". Moving down into an obstacle locks the piece.")]
    fn move_piece(
        &self,
        Parameters(req): Parameters<MoveRequest>,
    ) -> Result<CallToolResult, McpError> {
        let direction = match req.direction.to_lowercase().as_str() {
            "left" => Direction::Left,
            "right" => Direction::Right,
            "down" => Direction::Down,
            other => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Invalid direction: \"{other}\". Use \"left\", \"right\", or \"down\"."
                ))]));
            }
        };

        let mut game = self.game.lock().unwrap();
        let result = game.move_piece(direction);
        Ok(CallToolResult::success(vec![Content::text(
            format_move_result(result, &game, self),
        )]))
    }

    #[tool(description = "Rotate the active piece. Direction must be \"cw\" (clockwise) or \"ccw\" (counter-clockwise). Wall kicks are applied automatically.")]
    fn rotate(
        &self,
        Parameters(req): Parameters<RotateRequest>,
    ) -> Result<CallToolResult, McpError> {
        let dir = match req.direction.to_lowercase().as_str() {
            "cw" | "clockwise" => RotateDirection::Cw,
            "ccw" | "counterclockwise" | "counter-clockwise" => RotateDirection::Ccw,
            other => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Invalid rotation: \"{other}\". Use \"cw\" or \"ccw\"."
                ))]));
            }
        };

        let mut game = self.game.lock().unwrap();
        let result = game.rotate(dir);
        Ok(CallToolResult::success(vec![Content::text(
            format_move_result(result, &game, self),
        )]))
    }

    #[tool(description = "Instantly drop the piece to the bottom and lock it. Scores 2 points per row dropped. The next piece spawns immediately.")]
    fn hard_drop(&self) -> Result<CallToolResult, McpError> {
        let mut game = self.game.lock().unwrap();
        let result = game.hard_drop();
        Ok(CallToolResult::success(vec![Content::text(
            format_move_result(result, &game, self),
        )]))
    }

    #[tool(description = "Start a new game. The current game is saved automatically. Optionally pass a tag label to tag this game for comparison (e.g. \"baseline\", \"aggressive\", \"think-step-by-step\").")]
    fn new_game(
        &self,
        Parameters(req): Parameters<NewGameRequest>,
    ) -> Result<CallToolResult, McpError> {
        let mut game = self.game.lock().unwrap();
        self.save_game(&game);
        let seed: u64 = rand::random();
        let width = game.state().board.width();
        let height = game.state().board.height();
        *game = Game::new(width, height, seed);
        let tag = req.tag.as_deref().unwrap_or("default");
        Ok(CallToolResult::success(vec![Content::text(format!(
            "New game started (seed: {seed}, tag: \"{tag}\"). Board: {width}x{height}. Call get_instructions if you need a refresher."
        ))]))
    }

    #[tool(description = "Get the current score, lines cleared, level, and pieces placed.")]
    fn get_score(&self) -> Result<CallToolResult, McpError> {
        let game = self.game.lock().unwrap();
        let state = game.state();
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Score: {}\nLines cleared: {}\nLevel: {}\nPieces placed: {}\nGame over: {}",
            state.score, state.lines_cleared, state.level, state.pieces_placed, state.game_over
        ))]))
    }
}

const FLUSH_EVERY_N_MOVES: u32 = 10;

fn format_move_result(result: MoveResult, game: &Game, server: &TetrisServer) -> String {
    if result == MoveResult::GameOver {
        server.save_game(game);
    } else if game.history().metadata.total_moves % FLUSH_EVERY_N_MOVES == 0 {
        server.save_game(game);
    }
    let state = game.state();
    match result {
        MoveResult::Moved => format!(
            "Moved. Piece at ({}, {}), rotation {:?}",
            state.current_piece.x, state.current_piece.y, state.current_piece.rotation
        ),
        MoveResult::Locked { lines_cleared } => {
            let mut msg = format!("Piece locked. Score: {}", state.score);
            if lines_cleared > 0 {
                msg.push_str(&format!(". {} line(s) cleared!", lines_cleared));
            }
            msg.push_str(&format!(
                ". Next piece: {:?} at ({}, {})",
                state.current_piece.piece_type, state.current_piece.x, state.current_piece.y
            ));
            msg
        }
        MoveResult::GameOver => format!("GAME OVER! Final score: {}", state.score),
        MoveResult::Invalid => "Invalid move — piece can't go there.".to_string(),
    }
}

impl TetrisServer {
    pub fn new(width: usize, height: usize, seed: u64, games_dir: String) -> Self {
        Self {
            game: std::sync::Arc::new(Mutex::new(Game::new(width, height, seed))),
            games_dir,
            tool_router: Self::tool_router(),
        }
    }

    fn save_game(&self, game: &Game) {
        let history = game.history();
        if history.moves.is_empty() {
            return;
        }
        let _ = std::fs::create_dir_all(&self.games_dir);
        let path = format!("{}/{}.json", self.games_dir, history.metadata.game_id);
        match std::fs::write(&path, history.to_json()) {
            Ok(_) => tracing::info!("Game saved to {path}"),
            Err(e) => tracing::error!("Failed to save game: {e}"),
        }
    }
}

#[tool_handler]
impl ServerHandler for TetrisServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("tetris-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "You are playing Tetris! Call get_instructions first to learn the rules and available tools.".to_string(),
            )
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let args = Args::parse();
    let seed = args.seed.unwrap_or_else(|| rand::random());

    tracing::info!("Starting Tetris MCP server (seed: {seed}, board: {}x{})", args.width, args.height);

    let games_dir = args.games_dir.unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        format!("{home}/.tetris-mcp/games")
    });

    let server = TetrisServer::new(args.width, args.height, seed, games_dir);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
