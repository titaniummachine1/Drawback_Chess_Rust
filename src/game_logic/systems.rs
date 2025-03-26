use bevy::prelude::*;
use shakmaty::{Color as ChessColor, Position};
use crate::game_logic::state::{GameState, TurnState};
use crate::game_logic::events::{MakeMoveEvent, GameOverEvent};

/// System to apply a move to the game state
pub fn apply_move(
    _commands: Commands,
    mut ev_make_move: EventReader<MakeMoveEvent>,
    _ev_game_over: EventWriter<GameOverEvent>,
    mut game_state: ResMut<GameState>,
    mut next_state: ResMut<NextState<TurnState>>,
) {
    for ev in ev_make_move.read() {
        let move_to_make = ev.0.clone();
        let from_square = move_to_make.from().unwrap();
        let _to_square = move_to_make.to();
        
        // Check if piece exists at from_square
        let board = game_state.board.board();
        let piece_opt = board.piece_at(from_square);
        
        if let Some(_piece) = piece_opt {
            // Clone the current board state and apply the move
            let mut new_board = game_state.board.clone();
            new_board.play_unchecked(&move_to_make);
            
            // Update the game state with the new board
            game_state.board = new_board;
            
            // Update turn state
            game_state.current_player_turn = !game_state.current_player_turn;
            
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