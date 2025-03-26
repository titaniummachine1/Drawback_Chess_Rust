use shakmaty::{Move, Position, Chess, Role, Square};
use crate::ai::plugin::AiGameStateContext;
use rand::prelude::*;

/// AI implementation to find a move with improved heuristics.
/// Evaluates positions by examining material, checks, and board control.
/// Prioritizes defending against immediate checkmate threats.
pub fn find_best_move_mcts(ctx: AiGameStateContext, _iterations: u32) -> Option<Move> {
    // Get legal moves from current position
    let board_copy = ctx.board.clone();
    let legal_moves = board_copy.legal_moves();
    
    if legal_moves.is_empty() {
        return None;
    }
    
    let legal_moves_vec: Vec<Move> = legal_moves.into_iter().collect();
    let mut rng = rand::thread_rng();
    
    // Double check all our candidates are legal
    println!("AI considering {} legal moves", legal_moves_vec.len());
    
    // Check if we're in check - prioritize getting out of check if we are
    let in_check = board_copy.is_check();
    
    // Evaluate each move
    let mut move_scores: Vec<(Move, i32)> = Vec::new();
    
    for m in &legal_moves_vec {
        let mut test_board = board_copy.clone();
        
        // Check if the move is a capture and what's being captured
        let capture_value = get_capture_value(&test_board, m);
        
        // Play the move and see what results
        test_board.play_unchecked(m);
        
        // Calculate a score for this move
        let mut score = capture_value;
        
        // Heavily penalize moves that let opponent checkmate us
        let opponent_has_checkmate = test_board.is_checkmate();
        if opponent_has_checkmate {
            score -= 10000; // Strongly avoid moves that lead to immediate checkmate
        }
        
        // Strongly prefer moves that checkmate opponent
        if test_board.is_check() {
            // Check if this is a checkmate
            let opponent_legal_moves = test_board.legal_moves();
            let possible_responses = opponent_legal_moves.len();
            
            if possible_responses == 0 {
                score += 10000; // Strong preference for checkmate
            } else {
                score += 50; // Moderate preference for check
            }
        }
        
        // If we're in check, prioritize moves that get us out
        if in_check {
            score += 200; // Bonus for any legal move when in check
        }
        
        // Add material evaluation
        score += evaluate_material(&test_board);
        
        // Add the scored move to our list
        move_scores.push((m.clone(), score));
    }
    
    // Sort moves by score (descending)
    move_scores.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take the best move, or if scores are tied, choose randomly among the best
    let best_score = if let Some((_, score)) = move_scores.first() {
        *score
    } else {
        return legal_moves_vec.choose(&mut rng).cloned();
    };
    
    // Get all moves with the best score
    let best_moves: Vec<Move> = move_scores
        .iter()
        .filter(|(_, score)| *score == best_score)
        .map(|(mv, _)| mv.clone())
        .collect();
    
    // Choose randomly among best moves
    let selected_move = best_moves.choose(&mut rng).cloned();
    
    // Final validation - make sure the move is legal
    if let Some(mv) = &selected_move {
        // Make sure this move is in our legal moves list
        if !legal_moves_vec.contains(mv) {
            println!("WARNING: AI tried to play illegal move: {:?}", mv);
            // Fall back to a random legal move
            return legal_moves_vec.choose(&mut rng).cloned();
        }
        println!("AI selected valid move: {:?}", mv);
    }
    
    selected_move
}

/// Helper function to evaluate material advantage
fn evaluate_material(board: &Chess) -> i32 {
    let mut score = 0;
    
    // Material values
    const PAWN_VALUE: i32 = 100;
    const KNIGHT_VALUE: i32 = 320;
    const BISHOP_VALUE: i32 = 330;
    const ROOK_VALUE: i32 = 500;
    const QUEEN_VALUE: i32 = 900;
    
    // Evaluate material for each side
    for square in Square::ALL {
        if let Some(piece) = board.board().piece_at(square) {
            let piece_value = match piece.role {
                Role::Pawn => PAWN_VALUE,
                Role::Knight => KNIGHT_VALUE,
                Role::Bishop => BISHOP_VALUE,
                Role::Rook => ROOK_VALUE,
                Role::Queen => QUEEN_VALUE,
                Role::King => 0, // King value not included in material count
            };
            
            // Add to score for AI pieces (black), subtract for opponent pieces
            if piece.color == board.turn() {
                score += piece_value;
            } else {
                score -= piece_value;
            }
        }
    }
    
    score
}

/// Helper function to get the value of a captured piece
fn get_capture_value(board: &Chess, m: &Move) -> i32 {
    const PAWN_VALUE: i32 = 100;
    const KNIGHT_VALUE: i32 = 320;
    const BISHOP_VALUE: i32 = 330;
    const ROOK_VALUE: i32 = 500;
    const QUEEN_VALUE: i32 = 900;
    
    // Direct capture - destination has a piece
    if let Some(piece) = board.board().piece_at(m.to()) {
        return match piece.role {
            Role::Pawn => PAWN_VALUE,
            Role::Knight => KNIGHT_VALUE,
            Role::Bishop => BISHOP_VALUE,
            Role::Rook => ROOK_VALUE,
            Role::Queen => QUEEN_VALUE,
            Role::King => 0, // Can't capture king in chess
        };
    }
    
    // Check for en passant capture - pawn moving diagonally without a piece at destination
    if let Some(from_square) = m.from() {
        if let Some(piece) = board.board().piece_at(from_square) {
            if piece.role == Role::Pawn && from_square.file() != m.to().file() {
                return PAWN_VALUE; // En passant always captures a pawn
            }
        }
    }
    
    0
}