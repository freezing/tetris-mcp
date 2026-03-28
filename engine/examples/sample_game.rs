use engine::{Game, Direction, RotateDirection, MoveResult};

fn main() {
    let mut game = Game::new(10, 20, 42);

    // Play a simple game: hard drop pieces with some movement
    let mut move_count = 0;
    while !game.state().game_over && move_count < 200 {
        // Simple strategy: randomly move and rotate, then hard drop
        let piece_type = game.state().current_piece.piece_type;

        // Move based on piece type for variety
        match piece_type {
            engine::PieceType::I => { game.rotate(RotateDirection::Cw); game.move_piece(Direction::Left); }
            engine::PieceType::O => { game.move_piece(Direction::Right); }
            engine::PieceType::T => { game.rotate(RotateDirection::Cw); }
            engine::PieceType::S => { game.move_piece(Direction::Left); game.move_piece(Direction::Left); }
            engine::PieceType::Z => { game.move_piece(Direction::Right); game.move_piece(Direction::Right); }
            engine::PieceType::J => { game.rotate(RotateDirection::Ccw); game.move_piece(Direction::Left); }
            engine::PieceType::L => { game.rotate(RotateDirection::Cw); game.move_piece(Direction::Right); }
        }

        game.hard_drop();
        move_count += 1;
    }

    let history = game.history();
    println!("{}", history.to_json());

    eprintln!("Game over: {}", game.state().game_over);
    eprintln!("Score: {}", game.state().score);
    eprintln!("Lines cleared: {}", game.state().lines_cleared);
    eprintln!("Pieces placed: {}", game.state().pieces_placed);
    eprintln!("Total moves: {}", history.moves.len());
}
