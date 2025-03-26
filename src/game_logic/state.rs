use bevy::prelude::*;
use shakmaty::{Chess, Color as ChessColor};
use crate::drawbacks::registry::DrawbackId; // Use the ID enum

// Represents the overall status of the game
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus { Ongoing, GameOver }

// Bevy State to manage whose turn it is / what phase we are in
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum TurnState {
    #[default]
    PlayerTurn,
    AiTurn,
    ProcessingMove,
    GameOver,
}

/// Resource holding the primary chess game state.
#[derive(Resource)]
pub struct GameState {
    pub board: Chess, // Current board position
    pub current_player_turn: ChessColor,
    pub status: GameStatus,
    // --- Drawback State ---
    pub white_drawback: DrawbackId, // White's active drawback (None if no drawback)
    pub black_drawback: DrawbackId, // Black's active drawback (None if no drawback)
    // --- Per-Turn Randomness State ---
    // Stores the outcome of RNG generated *for the current player* at the start of their turn,
    // if their active drawback requires it. Cleared after the turn.
    pub current_turn_rng_outcome: Option<u8>,
     // --- Zobrist Hash ---
     // Placeholder: A proper Zobrist hash implementation is complex.
     // Add a field to store the hash, calculated elsewhere.
     pub zobrist_hash: u64, // The hash representing the current state

     // Add history Vec<MoveInfo> etc. later if needed
}

// Default implementation: Start with no drawbacks, standard board
impl Default for GameState {
    fn default() -> Self {
        Self {
            board: Chess::default(),
            current_player_turn: ChessColor::White,
            status: GameStatus::Ongoing,
            white_drawback: DrawbackId::None, // Start with no drawback
            black_drawback: DrawbackId::None, // Start with no drawback
            current_turn_rng_outcome: None,
             zobrist_hash: 0, // Initialize hash (will be calculated properly)
        }
    }
}

// Add methods to easily get the current player's drawback ID if needed
impl GameState {
    pub fn get_current_player_drawback_id(&self) -> DrawbackId {
         match self.current_player_turn {
             ChessColor::White => self.white_drawback,
             ChessColor::Black => self.black_drawback,
         }
    }
} 