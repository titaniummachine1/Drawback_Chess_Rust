use bevy::prelude::*;
use shakmaty::{Color as ChessColor, Position, Role, Square, Move};
use crate::game_logic::state::{GameState, TurnState, GameStatus};
use crate::game_logic::events::{MakeMoveEvent, GameOverEvent};
use crate::drawbacks::{DrawbackRegistry, DrawbackId};

/// Check if a move captures the king (Drawback Chess win condition)
fn is_king_capture(board: &shakmaty::Chess, m: &Move) -> bool {
    if let Some(piece) = board.board().piece_at(m.to()) {
        return piece.role == Role::King;
    }
    false
}

/// System to apply a move to the game state
pub fn apply_move(
    _commands: Commands,
    mut ev_make_move: EventReader<MakeMoveEvent>,
    mut ev_game_over: EventWriter<GameOverEvent>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<TurnState>>,
    current_state: Res<State<TurnState>>,
    drawback_registry: Res<DrawbackRegistry>,
) {
    for ev in ev_make_move.read() {
        let move_to_make = ev.0.clone();
        println!(">>> RECEIVED MOVE EVENT: {:?}", move_to_make);
        
        // Ensure we're only processing events in the correct turn state
        // This prevents the AI from making multiple moves
        let is_ai_move = game_state.current_player_turn == ChessColor::Black;
        let is_player_move = game_state.current_player_turn == ChessColor::White;
        
        if (is_ai_move && *current_state.get() != TurnState::AiTurn) || 
           (is_player_move && *current_state.get() != TurnState::PlayerTurn) {
            println!("!!! MOVE IGNORED: Wrong turn state for current player");
            continue;
        }
        
        // Check if move is legal ignoring check constraints (for Drawback Chess)
        let mut legal_moves = game_state.board.legal_moves();
        
        // Validate move against player's drawback constraints
        let player_drawback_id = game_state.get_current_player_drawback_id();
        if player_drawback_id != DrawbackId::None {
            if let Some(drawback_rule) = drawback_registry.rules.get(&player_drawback_id) {
                // Filter moves according to this player's drawback
                let moves_vec: Vec<Move> = legal_moves.into_iter().collect();
                legal_moves = drawback_rule.filter_pseudo_legal_moves(
                    &game_state.board,
                    moves_vec,
                    game_state.current_turn_rng_outcome
                ).into_iter().collect();
                println!("After applying drawback: {} legal moves", legal_moves.len());
            }
        }
        
        // In Drawback Chess, you're allowed to move into check if opponent's drawback prevents capture
        // For simplicity, we'll check if the move is in our legal moves list
        let move_is_allowed = legal_moves.contains(&move_to_make) || 
                              is_allowable_check_move(&game_state, &move_to_make);
        
        if !move_is_allowed {
            println!("!!! ILLEGAL MOVE ATTEMPTED: {:?}", move_to_make);
            continue;
        }
        
        println!("*** LEGAL MOVE CONFIRMED: {:?}", move_to_make);
        
        // Log the move
        if let Some(from_square) = move_to_make.from() {
            println!("Updating piece positions for move: {:?} ", move_to_make);
            
            // Verify the source square has a piece of the correct color
            if let Some(piece) = game_state.board.board().piece_at(from_square) {
                if piece.color != game_state.current_player_turn {
                    println!("!!! WRONG COLOR PIECE MOVE ATTEMPTED: {:?}", move_to_make);
                    continue;
                }
            } else {
                println!("Warning: Couldn't find piece at source square {:?}", from_square);
                continue;
            }
        }
        
        let to_square = move_to_make.to();
        
        // Check for capture - flag captures for future AI evaluation
        let is_capture = match game_state.board.board().piece_at(to_square) {
            Some(_) => true, // Destination square has a piece (standard capture)
            None => {
                // Check for en passant capture
                if let Some(from_square) = move_to_make.from() {
                    if let Some(piece) = game_state.board.board().piece_at(from_square) {
                        if piece.role == Role::Pawn && from_square.file() != to_square.file() {
                            true // Pawn moving diagonally without a piece at destination is en passant
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        };
        
        if is_capture {
            println!("Capture at square: {:?}", to_square);
        }
        
        // Check if this move captures the king (Drawback Chess win condition)
        let king_captured = is_king_capture(&game_state.board, &move_to_make);
        
        // Clone the current board state and apply the move
        let mut new_board = game_state.board.clone();
        new_board.play_unchecked(&move_to_make);
        
        // Check if the move results in check (just for convenience and AI logic)
        let is_check = new_board.is_check();
        if is_check {
            println!("Check!");
        }
        
        // In Drawback Chess, game ends when king is captured
        let is_game_over = king_captured;
        
        // Update the game state with the new board
        game_state.board = new_board;
        
        // Update turn state
        game_state.current_player_turn = !game_state.current_player_turn;
        
        // First, set to processing state to prevent double moves
        next_state.set(TurnState::ProcessingMove);
        
        // Handle game over conditions
        if is_game_over {
            game_state.status = GameStatus::GameOver;
            next_state.set(TurnState::GameOver);
            
            // Send game over event
            ev_game_over.send(GameOverEvent("King Captured".to_string()));
            
            println!("Game over: King Captured");
        } else {
            // Set next state based on current player
            if game_state.current_player_turn == ChessColor::Black {
                next_state.set(TurnState::AiTurn);
            } else {
                next_state.set(TurnState::PlayerTurn);
            }
        }
    }
}

/// Helper function to determine if a move into check might be allowable in Drawback Chess
/// based on opponent's potential drawback that might prevent king capture
fn is_allowable_check_move(game_state: &GameState, candidate_move: &Move) -> bool {
    // This is a simplified version - a full implementation would:
    // 1. Apply the move to a temporary board state
    // 2. Look ahead to see if the opponent would be able to capture the king
    // 3. Check the opponent's drawback to see if it blocks them from capturing the king
    // 
    // For now, we'll be conservative and not allow obviously illegal moves
    // In a complete implementation, we would need the opponent's drawback rule
    // to determine if king capture is prevented
    
    // Default implementation - be conservative, requiring strict legality
    false
}

// ... existing code ...