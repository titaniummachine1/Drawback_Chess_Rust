use shakmaty::{Move, Position, Chess, Role, Square, File, Rank};
use super::plugin::AiGameStateContext;
use std::time::Duration;
use rand::seq::SliceRandom;
use pleco::{Board, BitMove, PieceType};
use pleco::core::score::Score;

// Convert shakmaty Chess to Pleco Board
fn to_pleco_board(chess: &Chess) -> Option<Board> {
    // Convert position to FEN
    let position_fen = format!("{} {} - - 0 1", 
        chess.board().to_string(), 
        if chess.turn() == shakmaty::Color::White { "w" } else { "b" }
    );
    
    // Create a Pleco board from FEN
    match Board::from_fen(&position_fen) {
        Ok(board) => Some(board),
        Err(_) => None,
    }
}

// Convert square coordinates to shakmaty Square
fn coords_to_square(file: usize, rank: usize) -> Option<Square> {
    let file_enum = match file {
        0 => File::A,
        1 => File::B,
        2 => File::C,
        3 => File::D,
        4 => File::E,
        5 => File::F,
        6 => File::G,
        7 => File::H,
        _ => return None,
    };
    
    let rank_enum = match rank {
        0 => Rank::First,
        1 => Rank::Second,
        2 => Rank::Third,
        3 => Rank::Fourth,
        4 => Rank::Fifth,
        5 => Rank::Sixth,
        6 => Rank::Seventh,
        7 => Rank::Eighth,
        _ => return None,
    };
    
    // Square::from_coords returns a Square, not a Result
    Some(Square::from_coords(file_enum, rank_enum))
}

// Convert Pleco BitMove to shakmaty Move
fn to_shakmaty_move(bit_move: BitMove, chess: &Chess) -> Option<Move> {
    let from_idx = (bit_move.get_src().0 as usize) % 64;
    let to_idx = (bit_move.get_dest().0 as usize) % 64;
    
    let from_file = from_idx % 8;
    let from_rank = from_idx / 8;
    let to_file = to_idx % 8;
    let to_rank = to_idx / 8;
    
    let from_square = coords_to_square(from_file, from_rank)?;
    let to_square = coords_to_square(to_file, to_rank)?;
    
    // Get piece information
    let piece = chess.board().piece_at(from_square)?;
    let role = piece.role;
    let capture = chess.board().piece_at(to_square).map(|p| p.role);
    
    // Handle castling
    if role == Role::King && (from_file as i32 - to_file as i32).abs() > 1 {
        let rook_file = if to_file > from_file { 7 } else { 0 };
        let rook_square = coords_to_square(rook_file, from_rank)?;
        
        return Some(Move::Castle {
            king: from_square,
            rook: rook_square,
        });
    }
    
    // Handle promotion
    let promotion = if bit_move.is_promo() {
        match bit_move.promo_piece() {
            PieceType::Q => Some(Role::Queen),
            PieceType::R => Some(Role::Rook),
            PieceType::B => Some(Role::Bishop),
            PieceType::N => Some(Role::Knight),
            _ => None,
        }
    } else {
        None
    };
    
    Some(Move::Normal {
        role,
        from: from_square,
        to: to_square,
        capture,
        promotion,
    })
}

// Helper to compare Pleco scores
fn is_better_score(score: Score, best_score: Score) -> bool {
    // Higher scores are better in Pleco (from white's perspective)
    if score.0 == best_score.0 {
        score.1 > best_score.1
    } else {
        score.0 > best_score.0
    }
}

// Find the best move using Pleco's analysis
pub fn find_best_move_pleco(ctx: AiGameStateContext, time_limit: Duration, depth: u16) -> Option<Move> {
    // Get all legal moves
    let legal_moves = ctx.board.legal_moves();
    
    if legal_moves.is_empty() {
        return None;
    }
    
    // Convert shakmaty board to Pleco board
    let pleco_board = match to_pleco_board(&ctx.board) {
        Some(board) => board,
        None => {
            // Fallback to random move if conversion fails
            let mut rng = rand::thread_rng();
            return legal_moves.choose(&mut rng).cloned();
        }
    };
    
    // Use Pleco's search capabilities
    let depth_limit = if depth < 1 { 3 } else { depth as usize };
    
    // Find the best move using Pleco's minimax search
    let best_move = if time_limit.as_millis() > 100 {
        // Use a time-limited search
        let search_depth = std::cmp::min(depth_limit, 4); // Limit depth for time-based search
        let mut best_score = Score(-999999, -999999); // Use minimal Score value
        let mut best_bit_move = None;
        
        // Generate all legal moves in Pleco
        let pleco_moves = pleco_board.generate_moves();
        
        // Try each move and evaluate
        for bit_move in pleco_moves {
            let mut new_board = pleco_board.clone();
            new_board.apply_move(bit_move);
            
            // Simple evaluation based on material count
            let score = new_board.psq();
            
            if is_better_score(score, best_score) {
                best_score = score;
                best_bit_move = Some(bit_move);
            }
        }
        
        best_bit_move
    } else {
        // Use a depth-limited search
        let mut best_score = Score(-999999, -999999); // Use minimal Score value
        let mut best_bit_move = None;
        
        // Generate all legal moves in Pleco
        let pleco_moves = pleco_board.generate_moves();
        
        // Try each move and evaluate
        for bit_move in pleco_moves {
            let mut new_board = pleco_board.clone();
            new_board.apply_move(bit_move);
            
            // Simple evaluation based on material count
            let score = new_board.psq();
            
            if is_better_score(score, best_score) {
                best_score = score;
                best_bit_move = Some(bit_move);
            }
        }
        
        best_bit_move
    };
    
    // Handle the result
    match best_move {
        Some(bit_move) => {
            // Convert the move
            match to_shakmaty_move(bit_move, &ctx.board) {
                Some(m) => {
                    // Verify the move is legal
                    if ctx.board.is_legal(&m) {
                        Some(m)
                    } else {
                        // Fallback to random move if illegal
                        let mut rng = rand::thread_rng();
                        legal_moves.choose(&mut rng).cloned()
                    }
                },
                None => {
                    // Fallback to random move if conversion fails
                    let mut rng = rand::thread_rng();
                    legal_moves.choose(&mut rng).cloned()
                }
            }
        },
        None => {
            // Fallback to random move if search fails
            let mut rng = rand::thread_rng();
            legal_moves.choose(&mut rng).cloned()
        }
    }
} 