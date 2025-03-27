use bevy::prelude::*;
use shakmaty::{fen::Fen, Chess, Color, Role, Move, CastlingMode, Position, Setup};
use crate::drawbacks::DrawbackId;
use crate::config::GameConfig;
use crate::drawbacks::registry::DrawbackRegistry;
use crate::constants::DEFAULT_BOARD_FLIPPED;
use super::state::{GameState, TurnState, GameStatus};
use super::systems::apply_move;
use super::events::{MakeMoveEvent, GameOverEvent};

pub struct GameLogicPlugin;

/// The standard chess starting position FEN (including castling rights)
pub const STANDARD_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The flipped chess starting position FEN (black at bottom, white at top)
pub const FLIPPED_FEN: &str = "RNBQKBNR/PPPPPPPP/8/8/8/8/pppppppp/rnbqkbnr w - - 0 1";

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_state::<TurnState>()
            .add_event::<MakeMoveEvent>()
            .add_event::<GameOverEvent>()
            .add_systems(Startup, init_game_state)
            .add_systems(
                Update,
                apply_move.run_if(in_state(TurnState::PlayerTurn).or_else(in_state(TurnState::AiTurn)))
            );
    }
}

/// Initialize the game state with the starting position and turn
fn init_game_state(
    mut commands: Commands,
    config: Res<GameConfig>,
    drawback_registry: Res<DrawbackRegistry>,
    zobrist_keys: Res<crate::ai::zobrist::ZobristKeys>,
) {
    // Use the standard chess position FEN instead of the flipped one
    // This matches the visual representation (white at bottom, black at top)
    let fen = Fen::from_ascii(STANDARD_FEN.as_bytes()).expect("Valid FEN");
    let chess: Chess = fen.into_position(CastlingMode::Standard).expect("Valid position");
    
    // Initialize the GameState with default drawbacks from config
    let white_drawback_id = config.resolve_drawback_id(&config.white_player.drawback);
    let black_drawback_id = config.resolve_drawback_id(&config.black_player.drawback);
    
    println!("Initializing game with drawbacks - White: {:?}, Black: {:?}", 
             white_drawback_id, black_drawback_id);
    
    // Initialize and insert the GameState resource
    let mut game_state = GameState {
        board: chess,
        current_player_turn: Color::White,
        white_drawback: white_drawback_id,
        black_drawback: black_drawback_id,
        zobrist_hash: 0,  // Will be initialized properly
        status: GameStatus::Ongoing,
        current_turn_rng_outcome: None,
        board_flipped: DEFAULT_BOARD_FLIPPED,
    };

    // Update the zobrist hash with the initial position
    let hash = crate::ai::zobrist::calculate_zobrist_hash_for_board(&game_state.board, &zobrist_keys);
    game_state.zobrist_hash = hash;
    
    // Insert the initialized GameState as a resource
    commands.insert_resource(game_state);
}

// Check if a move captures the king (used for Drawback Chess win condition)
pub fn is_king_capture(board: &Chess, game_move: &Move) -> bool {
    if let Some(piece) = board.board().piece_at(game_move.to()) {
        return piece.role == Role::King;
    }
    false
}

fn setup_drawbacks(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    config: Res<GameConfig>,
) {
    println!("Setting up drawbacks from configuration...");
    
    // Get drawback IDs from configuration
    let white_drawback_id = config.resolve_drawback_id(&config.white_player.drawback);
    let black_drawback_id = config.resolve_drawback_id(&config.black_player.drawback);
    
    // Set drawbacks in game state
    game_state.white_drawback = white_drawback_id;
    game_state.black_drawback = black_drawback_id;
} 