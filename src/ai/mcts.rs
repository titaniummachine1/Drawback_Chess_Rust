use shakmaty::{Move, Position, Chess, Role, Square, Color, Piece, Outcome};
use shakmaty::fen::Fen;
use arrayvec::ArrayVec;
use std::cmp;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::drawbacks::DrawbackId;
use crate::drawbacks::registry::DrawbackRegistry;
use super::plugin::AiGameStateContext;
use super::evaluation::{evaluate_position_with_pst, compute_game_phase};
use rand::prelude::*;

/// AI implementation to find a move with improved heuristics.
/// Evaluates positions by examining material, checks, and board control.
/// Prioritizes capturing the king or defending against king captures.
pub fn find_best_move_mcts(ctx: AiGameStateContext, iterations: u32) -> Option<Move> {
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
        
        // Check if we're capturing a king (immediate win in Drawback Chess)
        let capturing_king = is_king_capture(&test_board, m);
        
        // Play the move and see what results
        test_board.play_unchecked(m);
        
        // Calculate a score for this move
        let mut score = capture_value;
        
        // Heavily prioritize capturing the king (immediate win in Drawback Chess)
        if capturing_king {
            score += 20000; // Extremely high value for king capture
        }

        // Check if opponent can capture our king after this move
        // To do this, we need to check all opponent responses
        let opponent_responses = test_board.legal_moves();
        let mut opponent_can_capture_king = false;
        
        for opp_move in opponent_responses.iter() {
            if is_king_capture(&test_board, opp_move) {
                // Opponent can capture our king on their next move - very bad!
                opponent_can_capture_king = true;
                break;
            }
        }
        
        if opponent_can_capture_king {
            score -= 15000; // Strongly avoid moves that allow opponent to capture our king
        }
        
        // Prefer moves that put opponent in check
        if test_board.is_check() {
            // We're checking the opponent
            let opponent_legal_moves = test_board.legal_moves();
            let possible_responses = opponent_legal_moves.len();
            
            // The fewer responses, the better for us
            score += 50 + (20 * (20 - possible_responses as i32)).max(0); 
        }
        
        // If we're in check, prioritize moves that get us out
        if in_check {
            score += 300; // Bonus for any legal move when in check
        }
        
        // Add evaluation using piece-square tables
        let pst_score = evaluate_position_with_pst(&test_board);
        println!("Move {:?} - PST evaluation: {}", m, pst_score);
        score += pst_score;
        
        // Add the scored move to our list
        move_scores.push((m.clone(), score));
    }
    
    // Sort moves by score (descending)
    move_scores.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Print the top 3 best moves with their scores for debugging
    println!("Top move evaluations:");
    for (i, (mv, score)) in move_scores.iter().take(3).enumerate() {
        println!("  {}. {:?} - Score: {}", i+1, mv, score);
    }
    
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
    const PAWN_VALUE: i32 = 100;
    const KNIGHT_VALUE: i32 = 320;
    const BISHOP_VALUE: i32 = 330;
    const ROOK_VALUE: i32 = 500;
    const QUEEN_VALUE: i32 = 900;
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

const WIN_SCORE: f64 = 1.0;
const DRAW_SCORE: f64 = 0.5;
const LOSE_SCORE: f64 = 0.0;
const EXPLORATION_CONSTANT: f64 = 1.4;

#[derive(Clone)]
struct MCTSNode {
    visits: u32,
    score: f64,
    position: Chess,
    move_made: Option<Move>,
    children: Vec<MCTSNode>,
    unexplored_moves: Vec<Move>,
    current_hash: u64,
    has_expanded: bool,
}

impl MCTSNode {
    fn new(position: Chess, move_made: Option<Move>, current_hash: u64) -> Self {
        Self {
            visits: 0,
            score: 0.0,
            position,
            move_made,
            children: Vec::new(),
            unexplored_moves: Vec::new(),
            current_hash,
            has_expanded: false,
        }
    }

    fn expand(&mut self, player_drawback: DrawbackId, opponent_drawback: DrawbackId, turn_color: Color) {
        if !self.has_expanded {
            let legal_moves = self.position.legal_moves();
            self.unexplored_moves = legal_moves.into_iter().collect();
            self.has_expanded = true;
        }
    }

    fn best_child(&self, exploration: bool) -> Option<usize> {
        if self.children.is_empty() {
            return None;
        }

        let mut best_score = f64::NEG_INFINITY;
        let mut best_indices = Vec::new();

        for (i, child) in self.children.iter().enumerate() {
            let exploitation = child.score / child.visits as f64;
            
            let exploration_score = if exploration {
                EXPLORATION_CONSTANT * ((self.visits as f64).ln() / child.visits as f64).sqrt()
            } else {
                0.0
            };

            let score = exploitation + exploration_score;

            if score > best_score {
                best_score = score;
                best_indices.clear();
                best_indices.push(i);
            } else if score == best_score {
                best_indices.push(i);
            }
        }

        // In case of a tie, select randomly from the best
        if !best_indices.is_empty() {
            let idx = if best_indices.len() > 1 {
                *best_indices.choose(&mut thread_rng()).unwrap()
            } else {
                best_indices[0]
            };
            Some(idx)
        } else {
            None
        }
    }
}

/// Evaluates a position from the AI's perspective
fn evaluate_position(position: &Chess, player_drawback: DrawbackId, opponent_drawback: DrawbackId, player_is_white: bool) -> f64 {
    // Handle terminal positions: checkmate, stalemate, etc.
    if let Some(outcome) = position.outcome() {
        match outcome {
            Outcome::Decisive { winner } => {
                let is_winner = (winner == Color::White && player_is_white) 
                                 || (winner == Color::Black && !player_is_white);
                return if is_winner { WIN_SCORE } else { LOSE_SCORE };
            }
            Outcome::Draw => return DRAW_SCORE,
        }
    }
    
    // Check if position is checkmate after our move
    if position.is_checkmate() {
        // We just checkmated the opponent, great!
        return WIN_SCORE;
    }
    
    // Check if we're in check after making this move
    if position.is_check() {
        // Our move puts/leaves us in check, not great
        return 0.2;
    }
    
    // Material evaluation
    let mut score = 0.0;
    
    // Count material
    let board = position.board();
    
    // Piece values: pawn=1, knight=3, bishop=3, rook=5, queen=9
    for square in Square::ALL {
        if let Some(piece) = board.piece_at(square) {
            let piece_value = match piece.role {
                Role::Pawn => 1.0,
                Role::Knight => 3.0,
                Role::Bishop => 3.0,
                Role::Rook => 5.0,
                Role::Queen => 9.0,
                Role::King => 0.0, // King is not counted in material score
            };
            
            score += if (piece.color == Color::White && player_is_white) 
                      || (piece.color == Color::Black && !player_is_white) {
                piece_value
            } else {
                -piece_value
            };
        }
    }
    
    // Normalize score to [0, 1] range
    let score_plus_offset = score + 40.0;
    let clamped_low = if score_plus_offset < 0.0 { 0.0 } else { score_plus_offset };
    let clamped_high = if clamped_low > 80.0 { 80.0 } else { clamped_low };
    let normalized_score = clamped_high / 80.0;
    
    // Return the score
    normalized_score
}

/// Checks if a position is "quiet" (no checks or captures available)
fn is_position_quiet(position: &Chess) -> bool {
    // A position is not quiet if it's in check
    if position.is_check() {
        return false;
    }
    
    // Check for possible captures
    let legal_moves = position.legal_moves();
    for m in legal_moves {
        // If there's a capture move available, position is not quiet
        if position.board().piece_at(m.to()) != None {
            return false;
        }
    }
    
    // No checks or captures, position is quiet
    true
}

/// Monte Carlo Tree Search implementation for finding the best move
pub fn find_best_move_mcts_monte_carlo(context: AiGameStateContext, iterations: u32) -> Option<Move> {
    // Start timing
    let start_time = Instant::now();
    let time_limit = Duration::from_millis(context.time_limit_ms as u64);
    
    // Ensure we're working with a copied board
    let board_copy = context.board.clone();
    let player_turn = context.player_turn;
    let player_is_white = player_turn == Color::White;
    
    // Create root node with the copied board 
    let mut root = MCTSNode::new(board_copy, None, context.current_hash);
    root.expand(context.player_drawback, context.opponent_drawback, player_turn);
    
    // Skip if no legal moves
    if root.unexplored_moves.is_empty() {
        return None;
    }
    
    // If only one legal move, return it immediately
    if root.unexplored_moves.len() == 1 {
        return Some(root.unexplored_moves[0].clone());
    }
    
    let mut completed_iterations = 0;
    
    for _ in 0..iterations {
        // Check if time limit is reached
        if start_time.elapsed() >= time_limit {
            println!("MCTS stopped after {} iterations due to time limit", completed_iterations);
            break;
        }
        
        // Selection and Expansion phases - simplified to avoid borrow checker issues
        let mut current_path = Vec::new();
        let mut current_node = &mut root;
        
        // Selection phase - visit nodes until we find a leaf
        loop {
            if current_node.unexplored_moves.is_empty() && !current_node.children.is_empty() {
                // Node is fully expanded but not a leaf, select best child
                if let Some(best_idx) = current_node.best_child(true) {
                    // Save the path we're taking
                    current_path.push(best_idx);
                    
                    // Navigate to the best child
                    let child_idx = best_idx;
                    // We can't keep a mutable reference while indexing, so we break and reindex later
                    break;
                } else {
                    // No valid child to select, treat as leaf
                    break;
                }
            } else {
                // Found a leaf node (either unexpanded or terminal)
                break;
            }
        }
        
        // Navigate to the selected leaf node using the path
        let mut current = &mut root;
        for &idx in current_path.iter() {
            current = &mut current.children[idx];
        }
        
        // Expansion phase
        if !current.unexplored_moves.is_empty() && current.visits > 0 {
            // Expand the current node by adding a child
            let random_index = thread_rng().gen_range(0..current.unexplored_moves.len());
            let new_move = current.unexplored_moves.swap_remove(random_index);
            
            // Create a new position by applying the selected move
            let mut new_position = current.position.clone();
            new_position.play_unchecked(&new_move);
            
            // Create a new child node
            let next_color = if player_turn == Color::White { Color::Black } else { Color::White };
            let mut new_child = MCTSNode::new(new_position, Some(new_move), context.current_hash);
            new_child.expand(context.opponent_drawback, context.player_drawback, next_color);
            
            // Add the new child to the current node
            current.children.push(new_child);
            
            // Update current_path to include the new child
            current_path.push(current.children.len() - 1);
            
            // Re-navigate to the new child node
            current = &mut root;
            for &idx in current_path.iter() {
                current = &mut current.children[idx];
            }
        }
        
        // Simulation phase
        let mut simulation_position = current.position.clone();
        
        // Determine whose turn it is in the simulation
        let mut simulation_turn = if current_path.len() % 2 == 0 { 
            player_turn
        } else {
            if player_turn == Color::White { Color::Black } else { Color::White }
        };
        
        let mut depth = 0;
        let max_depth = context.depth;
        
        while depth < max_depth {
            // Check if position is terminal
            if simulation_position.outcome().is_some() {
                break;
            }
            
            // If we're checking for quiet positions and depth > 0, check if position is quiet
            if context.check_quietness && depth > 0 && is_position_quiet(&simulation_position) {
                // If position is quiet, we can stop the simulation
                break;
            }
            
            // If depth reaches max but position isn't quiet, check if we should extend (quiescence search)
            if depth >= max_depth && !is_position_quiet(&simulation_position) && 
               depth < max_depth + context.quiescence_depth {
                // Continue with quiescence search
            } else if depth >= max_depth {
                // Stop at max depth
                break;
            }
            
            // Get legal moves
            let legal_moves = simulation_position.legal_moves();
            if legal_moves.is_empty() {
                break;
            }
            
            // Select a random move
            let random_move_idx = thread_rng().gen_range(0..legal_moves.len());
            let random_move = legal_moves[random_move_idx].clone();
            simulation_position.play_unchecked(&random_move);
            
            // Switch turns
            simulation_turn = if simulation_turn == Color::White { Color::Black } else { Color::White };
            
            depth += 1;
        }
        
        // Evaluate the final position
        let current_player_drawback = if simulation_turn == player_turn {
            context.player_drawback
        } else {
            context.opponent_drawback
        };
        
        let opponent_drawback = if simulation_turn == player_turn {
            context.opponent_drawback
        } else {
            context.player_drawback
        };
        
        let score = evaluate_position(&simulation_position, current_player_drawback, opponent_drawback, player_is_white);
        
        // Backpropagation phase - update all nodes along the path
        let mut current = &mut root;
        current.visits += 1;
        current.score += score;
        
        for &idx in current_path.iter() {
            current = &mut current.children[idx];
            current.visits += 1;
            current.score += score;
        }
        
        completed_iterations += 1;
    }
    
    println!("MCTS completed {} iterations in {:?}", completed_iterations, start_time.elapsed());
    
    // Select the best child from the root
    if let Some(best_child_idx) = root.best_child(false) {
        let best_child = &root.children[best_child_idx];
        println!("Best move score: {}/{} = {}", 
            best_child.score, 
            best_child.visits, 
            best_child.score / best_child.visits as f64);
        return best_child.move_made.clone();
    }
    
    // If no best child found but we have unexplored moves, return a random one
    if !root.unexplored_moves.is_empty() {
        let random_index = thread_rng().gen_range(0..root.unexplored_moves.len());
        return Some(root.unexplored_moves[random_index].clone());
    }
    
    None
}