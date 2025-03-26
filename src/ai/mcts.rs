use shakmaty::{Move, Position, Chess, Role};
use crate::ai::plugin::AiGameStateContext;
use rand::prelude::*;

/// Simple placeholder AI implementation to find a move with basic heuristics.
/// Prefers moves that result in check or capture, then random legal moves.
/// Returns the best move found.
pub fn find_best_move_mcts(ctx: AiGameStateContext, _iterations: u32) -> Option<Move> {
    // Get legal moves from current position
    let mut board_copy = ctx.board.clone();
    let legal_moves = board_copy.legal_moves();
    
    if legal_moves.is_empty() {
        return None;
    }
    
    let legal_moves_vec: Vec<Move> = legal_moves.into_iter().collect();
    let mut rng = rand::thread_rng();
    
    // Check if any move leads to immediate check
    let mut check_moves = Vec::new();
    // Check if any move is a capture
    let mut capture_moves = Vec::new();
    
    for m in &legal_moves_vec {
        let mut test_board = board_copy.clone();
        
        // Check if the move is a capture
        let is_capture = is_capturing_move(&test_board, m);
        if is_capture {
            capture_moves.push(m.clone());
        }
        
        // Play the move and see if it results in check
        test_board.play_unchecked(m);
        if test_board.is_check() {
            check_moves.push(m.clone());
        }
    }
    
    // Prioritize moves that give check AND capture
    let mut check_and_capture = Vec::new();
    for m in &check_moves {
        if capture_moves.contains(m) {
            check_and_capture.push(m.clone());
        }
    }
    
    // Return a move in order of priority:
    // 1. Moves that both check and capture
    // 2. Moves that check
    // 3. Moves that capture
    // 4. Any random legal move
    if !check_and_capture.is_empty() {
        return check_and_capture.choose(&mut rng).cloned();
    } else if !check_moves.is_empty() {
        return check_moves.choose(&mut rng).cloned();
    } else if !capture_moves.is_empty() {
        return capture_moves.choose(&mut rng).cloned();
    } else {
        return legal_moves_vec.choose(&mut rng).cloned();
    }
}

/// Helper function to check if a move captures a piece
fn is_capturing_move(board: &Chess, m: &Move) -> bool {
    // Direct capture - destination has a piece
    if let Some(to_square) = board.board().piece_at(m.to()) {
        return true;
    }
    
    // Check for en passant capture - pawn moving diagonally without a piece at destination
    if let Some(from_square) = m.from() {
        if let Some(piece) = board.board().piece_at(from_square) {
            if piece.role == Role::Pawn && from_square.file() != m.to().file() {
                return true; // Pawn moving diagonally is either a capture or en passant
            }
        }
    }
    
    false
}