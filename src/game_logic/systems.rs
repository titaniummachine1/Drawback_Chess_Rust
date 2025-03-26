use bevy::prelude::*;
use shakmaty::{Color as ChessColor, Position, Role, Square, Move};
use crate::game_logic::state::{GameState, TurnState, GameStatus};
use crate::game_logic::events::{MakeMoveEvent, GameOverEvent};

/// System to apply a move to the game state
pub fn apply_move(
    _commands: Commands,
    mut ev_make_move: EventReader<MakeMoveEvent>,
    mut ev_game_over: EventWriter<GameOverEvent>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    for ev in ev_make_move.read() {
        let move_to_make = ev.0.clone();
        
        // Check if move is legal
        let legal_moves = game_state.board.legal_moves();
        if !legal_moves.contains(&move_to_make) {
            println!("Illegal move attempted: {:?}", move_to_make);
            continue;
        }
        
        // Log the move
        let from_square = move_to_make.from().unwrap_or(Square::A1); // Some special moves might not have from square
        let to_square = move_to_make.to();
        
        // Check for capture - flag captures for future AI evaluation
        let is_capture = match game_state.board.board().piece_at(to_square) {
            Some(_) => true, // Destination square has a piece (standard capture)
            None => {
                // Check for en passant capture
                if let Some(piece) = game_state.board.board().piece_at(from_square) {
                    if piece.role == Role::Pawn && from_square.file() != to_square.file() {
                        true // Pawn moving diagonally without a piece at destination is en passant
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
        
        // Clone the current board state and apply the move
        let mut new_board = game_state.board.clone();
        new_board.play_unchecked(&move_to_make);
        
        // Check if the move results in check
        let is_check = new_board.is_check();
        if is_check {
            println!("Check!");
        }
        
        // Check for checkmate or stalemate
        let is_game_over = new_board.is_checkmate() || new_board.is_stalemate();
        
        // Update the game state with the new board
        game_state.board = new_board;
        
        // Update turn state
        game_state.current_player_turn = !game_state.current_player_turn;
        
        // Handle game over conditions
        if is_game_over {
            game_state.status = GameStatus::GameOver;
            next_state.set(TurnState::GameOver);
            
            // Send game over event
            ev_game_over.send(GameOverEvent(
                if is_check { "Checkmate".to_string() } else { "Stalemate".to_string() }
            ));
            
            println!("Game over: {}", if is_check { "Checkmate" } else { "Stalemate" });
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

// ... existing code ...