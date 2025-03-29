use shakmaty::{Move, Position, Chess, Role, Square, Color, Outcome};
use std::time::{Duration, Instant};
use crate::drawbacks::DrawbackId;
use super::plugin::AiGameStateContext;
use super::evaluation::evaluate_position_with_pst;
use rand::prelude::*;

/// AI implementation to find a move with improved heuristics.
/// Evaluates positions by examining material, checks, and board control.
/// Prioritizes capturing the king or defending against king captures.
pub fn find_best_move_mcts(ctx: AiGameStateContext, iterations: u32) -> Option<Move> {
    // Initialize timing
    let start_time = Instant::now();
    let time_limit = Duration::from_millis(ctx.time_limit_ms as u64);
    println!("AI starting search with time limit: {}ms", ctx.time_limit_ms);

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
    
    // Initialize best move tracking
    let mut overall_best_moves: Vec<(Move, i32)> = Vec::new();
    let mut iterations_completed = 0;
    
    // Main search loop - continue until time limit or iteration limit
    while start_time.elapsed() < time_limit && iterations_completed < iterations {
        iterations_completed += 1;
        
        // For tracking progress
        if iterations_completed % 1000 == 0 {
            println!("Search progress: {} iterations, elapsed: {:?}", 
                     iterations_completed, start_time.elapsed());
        }
        
        // Evaluate each move
        let mut move_scores: Vec<(Move, i32)> = Vec::new();
        
        for m in &legal_moves_vec {
            let mut test_board = board_copy.clone();
            
            // Check if the move is a capture and what's being captured
            let capture_value = get_capture_value(&test_board, m);
            
            // Check if we're capturing a king (immediate win in Drawback Chess)
            let capturing_king = is_king_capture(&test_board, m);
            
            // Play the move and see what results
            test_board.play_unchecked(m);
            
            // Calculate a score for this move
            let mut score = capture_value * 3; // Significantly increase the value of captures
            
            // Heavily prioritize capturing the king (immediate win in Drawback Chess)
            if capturing_king {
                score += 20000; // Extremely high value for king capture
            }

            // Get whose turn it is (this is who we're playing as)
            let our_color = match test_board.turn() {
                Color::White => Color::Black, // If white to move next, we just moved as black
                Color::Black => Color::White, // If black to move next, we just moved as white
            };

            // Check if opponent can capture our king after this move
            // To do this, we need to check all opponent responses
            let opponent_responses = test_board.legal_moves();
            let mut opponent_can_capture_king = false;
            let mut our_pieces_at_risk = 0;
            let mut max_piece_value_at_risk = 0;
            
            for opp_move in opponent_responses.iter() {
                if is_king_capture(&test_board, opp_move) {
                    // Opponent can capture our king on their next move - very bad!
                    opponent_can_capture_king = true;
                    break;
                }
                
                // Check if opponent can capture any of our pieces
                let piece_value = get_capture_value(&test_board, opp_move);
                if piece_value > 0 {
                    our_pieces_at_risk += 1;
                    max_piece_value_at_risk = max_piece_value_at_risk.max(piece_value);
                }
            }
            
            if opponent_can_capture_king {
                score -= 15000; // Strongly avoid moves that allow opponent to capture our king
            }
            
            // Avoid hanging pieces - Penalize moves that leave our pieces undefended
            if our_pieces_at_risk > 0 {
                // More aggressive protection for white pieces to balance against black
                let piece_risk_penalty = if our_color == Color::White {
                    max_piece_value_at_risk * 2
                } else {
                    max_piece_value_at_risk * 2
                };
                score -= piece_risk_penalty;
            }
            
            // Prefer moves that put opponent in check
            if test_board.is_check() {
                // We're checking the opponent
                let opponent_legal_moves = test_board.legal_moves();
                let possible_responses = opponent_legal_moves.len();
                
                // The fewer responses, the better for us
                score += 70 + (30 * (20 - possible_responses as i32)).max(0); 
            }
            
            // If we're in check, prioritize moves that get us out
            if in_check {
                score += 500; // Higher bonus for escaping check
            }
            
            // Add king safety bonus, with extra protection for white's king
            score += evaluate_king_safety(&test_board, our_color);
            
            // Add evaluation using piece-square tables
            let pst_score = -evaluate_position_with_pst(&test_board);
            score += pst_score;
            
            // Add the scored move to our list
            move_scores.push((m.clone(), score));
        }
        
        // Sort moves by score (descending)
        move_scores.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Update overall best moves
        if overall_best_moves.is_empty() || 
           (move_scores.first().map(|(_, s)| *s).unwrap_or(0) > 
            overall_best_moves.first().map(|(_, s)| *s).unwrap_or(0)) {
            overall_best_moves = move_scores.clone();
        }
        
        // If we find a winning move, we can stop searching
        if move_scores.first().map(|(_, s)| *s).unwrap_or(0) > 10000 {
            println!("Found winning move after {} iterations", iterations_completed);
            break;
        }
    }
    
    println!("AI completed {} iterations in {:?}", iterations_completed, start_time.elapsed());
    
    // Print the top 3 best moves with their scores for debugging
    println!("Top move evaluations:");
    for (i, (mv, score)) in overall_best_moves.iter().take(3).enumerate() {
        println!("  {}. {:?} - Score: {}", i+1, mv, score);
        // Debug PST evaluation
        let mut test_board = board_copy.clone();
        test_board.play_unchecked(mv);
        let pst_score = -evaluate_position_with_pst(&test_board);
        println!("Move {:?} - PST evaluation: {}", mv, pst_score);
    }
    
    // Take the best move, or if scores are tied, choose randomly among the best
    let best_score = if let Some((_, score)) = overall_best_moves.first() {
        *score
    } else {
        return legal_moves_vec.choose(&mut rng).cloned();
    };
    
    // Get all moves with the best score
    let best_moves: Vec<Move> = overall_best_moves
        .iter()
        .filter(|(_, score)| *score == best_score)
        .map(|(mv, _)| mv.clone())
        .collect();
    
    println!("Found {} moves with best score {}", best_moves.len(), best_score);
    
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

/// Helper function to check if a move captures the king
fn is_king_capture(board: &Chess, m: &Move) -> bool {
    if let Some(piece) = board.board().piece_at(m.to()) {
        return piece.role == Role::King;
    }
    false
}

/// Helper function to get the value of a captured piece
fn get_capture_value(board: &Chess, m: &Move) -> i32 {
    const PAWN_VALUE: i32 = 120;
    const KNIGHT_VALUE: i32 = 370;
    const BISHOP_VALUE: i32 = 380;
    const ROOK_VALUE: i32 = 550;
    const QUEEN_VALUE: i32 = 1000;
    const KING_VALUE: i32 = 20000; // Very high value for king capture in Drawback Chess
    
    // Direct capture - destination has a piece
    if let Some(piece) = board.board().piece_at(m.to()) {
        return match piece.role {
            Role::Pawn => PAWN_VALUE,
            Role::Knight => KNIGHT_VALUE,
            Role::Bishop => BISHOP_VALUE,
            Role::Rook => ROOK_VALUE,
            Role::Queen => QUEEN_VALUE,
            Role::King => KING_VALUE, // Can capture king in Drawback Chess
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

/// Evaluate king safety based on surrounding pieces and pawn structure
fn evaluate_king_safety(board: &Chess, color: Color) -> i32 {
    let king_square = find_king_square(board, color);
    if king_square.is_none() {
        return 0; // No king found or already captured
    }
    
    let king_sq = king_square.unwrap();
    let mut safety_score = 0;
    
    // Check if king is in a corner or on edge (generally safer in early/mid game)
    let file = king_sq.file().char() as u8 - b'a';
    let rank = king_sq.rank().char() as u8 - b'1';
    
    // Bonus for king on edge or corner
    if file == 0 || file == 7 || rank == 0 || rank == 7 {
        safety_score += 20;
        
        // Extra bonus for corner
        if (file == 0 || file == 7) && (rank == 0 || rank == 7) {
            safety_score += 15;
        }
    }
    
    // Check for friendly pawns protecting the king
    for offset in &[(-1, 1), (0, 1), (1, 1), (-1, 0), (1, 0), (-1, -1), (0, -1), (1, -1)] {
        let new_file = file as i32 + offset.0;
        let new_rank = rank as i32 + offset.1;
        
        // Skip if out of bounds
        if new_file < 0 || new_file > 7 || new_rank < 0 || new_rank > 7 {
            continue;
        }
        
        // Convert to square using a direct approach
        let new_file_char = (new_file as u8 + b'a') as char;
        let new_rank_char = (new_rank as u8 + b'1') as char;
        
        // We'll just check all squares on the board
        for adj_square in Square::ALL {
            let square_file = adj_square.file().char();
            let square_rank = adj_square.rank().char();
            
            // Only consider the square we're interested in
            if square_file == new_file_char && square_rank == new_rank_char {
                // Check if friendly piece is on this square
                if let Some(piece) = board.board().piece_at(adj_square) {
                    if piece.color == color {
                        // Friendly piece is protecting king
                        let protection_bonus = match piece.role {
                            Role::Pawn => 15,    // Pawns are good shields
                            Role::Knight => 10,
                            Role::Bishop => 8,
                            Role::Rook => 12,
                            Role::Queen => 5,    // Queen should usually not be next to king
                            Role::King => 0,     // Another king? Shouldn't happen
                        };
                        safety_score += protection_bonus;
                    }
                }
                break; // Found the square we were looking for
            }
        }
    }
    
    // Give white king some extra safety bonus to balance black's advantage
    if color == Color::White {
        safety_score = (safety_score as f32 * 1.5) as i32; // 50% bonus for white king safety
    }
    
    safety_score
}

/// Find the king's square for a given color
fn find_king_square(board: &Chess, color: Color) -> Option<Square> {
    for square in Square::ALL {
        if let Some(piece) = board.board().piece_at(square) {
            if piece.role == Role::King && piece.color == color {
                return Some(square);
            }
        }
    }
    None
}