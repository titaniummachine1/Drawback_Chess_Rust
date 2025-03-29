use rand::seq::SliceRandom;
use shakmaty::{Move, Chess, Position};
use super::plugin::AiGameStateContext;

/// Find the best move using a simple random approach
pub fn find_best_move_pleco(ctx: AiGameStateContext, _iterations: u32) -> Option<Move> {
    // Get all legal moves from the shakmaty board
    let legal_moves = ctx.board.legal_moves();
    
    if legal_moves.is_empty() {
        return None;
    }
    
    // Select a random move
    let mut rng = rand::thread_rng();
    legal_moves.choose(&mut rng).cloned()
} 