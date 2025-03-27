use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures_lite::future;
use std::time::Duration;
use shakmaty::{Chess, Color as ChessColor, Move, Position};
use crate::game_logic::state::{GameState, TurnState};
use crate::game_logic::events::MakeMoveEvent;
use crate::drawbacks::{DrawbackRegistry, DrawbackId};
use crate::config::GameConfig;
use crate::constants::DEFAULT_BOARD_FLIPPED;
use super::components::AiThinking;
use super::mcts::find_best_move_mcts;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            (
                request_ai_move.run_if(in_state(TurnState::AiTurn)),
                check_ai_move_result.run_if(in_state(TurnState::AiTurn)),
            )
        );
    }
}

/// Represents the state of the AI's game analysis.
#[derive(Clone)]
pub struct AiGameStateContext {
    pub board: Chess,
    pub player_turn: ChessColor,
    pub player_drawback: DrawbackId,
    pub opponent_drawback: DrawbackId,
    pub current_hash: u64,
    pub depth: u8,              // Search depth limit
    pub check_quietness: bool,  // Whether to ensure positions are quiet
    pub quiescence_depth: u8,   // Extra depth for non-quiet positions
    pub time_limit_ms: u32,     // Time limit in milliseconds
}

impl AiGameStateContext {
    pub fn from_game_state(game_state: &GameState, config: &GameConfig) -> Self {
        Self {
            board: game_state.board.clone(),
            player_turn: game_state.current_player_turn,
            player_drawback: game_state.get_current_player_drawback_id(),
            opponent_drawback: match game_state.current_player_turn {
                ChessColor::White => game_state.black_drawback,
                ChessColor::Black => game_state.white_drawback,
            },
            current_hash: game_state.zobrist_hash,
            depth: config.ai_settings.depth_limit,
            check_quietness: config.ai_settings.check_quietness,
            quiescence_depth: config.ai_settings.quiescence_depth,
            time_limit_ms: config.ai_settings.time_limit_ms,
        }
    }
}

/// System to spawn the AI calculation task
fn request_ai_move(
    mut commands: Commands,
    game_state: Res<GameState>,
    config: Res<GameConfig>,
    drawback_registry: Res<DrawbackRegistry>,
    q_ai_task: Query<&AiThinking>,
) {
    if q_ai_task.is_empty() {
        println!("AI turn detected. Spawning calculation task...");

        let thread_pool = AsyncComputeTaskPool::get();

        // Create a deep copy of the game state for AI to use
        let game_state_copy = GameState {
            board: game_state.board.clone(),
            current_player_turn: game_state.current_player_turn,
            status: game_state.status,
            white_drawback: game_state.white_drawback,
            black_drawback: game_state.black_drawback,
            current_turn_rng_outcome: game_state.current_turn_rng_outcome,
            zobrist_hash: game_state.zobrist_hash,
            board_flipped: game_state.board_flipped,
        };

        // Determine current player and opponent drawback IDs
        let (player_id, opponent_id) = match game_state_copy.current_player_turn {
             ChessColor::White => (game_state_copy.white_drawback, game_state_copy.black_drawback),
             ChessColor::Black => (game_state_copy.black_drawback, game_state_copy.white_drawback),
        };

        // Create DrawbackRule instances if needed for the current game state
        // This helps the AI consider the effects of both players' drawbacks
        let _player_drawback_arc = if player_id != DrawbackId::None {
            Some(drawback_registry.rules.get(&player_id).cloned())
        } else {
            None
        };

        let _opponent_drawback_arc = if opponent_id != DrawbackId::None {
            Some(drawback_registry.rules.get(&opponent_id).cloned())
        } else {
            None
        };

        // Prepare data for the async task - using the copy
        let ai_context = AiGameStateContext::from_game_state(&game_state_copy, &config);
        let iterations = config.ai_settings.iteration_limit;

        // Spawn the Async Task
        let task = thread_pool.spawn(async move {
            find_best_move_mcts(ai_context, iterations)
        });

        commands.spawn(AiThinking(task));
        println!("AI calculation task spawned.");
    }
}

/// System to check the AiThinking task result
fn check_ai_move_result(
    mut commands: Commands,
    mut task_q: Query<(Entity, &mut AiThinking)>,
    mut ev_make_move: EventWriter<MakeMoveEvent>,
    game_state: Res<GameState>,
) {
    for (entity, mut ai_task) in task_q.iter_mut() {
        if let Some(result_move) = future::block_on(future::poll_once(&mut ai_task.0)) {
            println!("AI calculation task finished.");
            if let Some(ai_move) = result_move {
                // Validate the move before sending it
                let is_valid = validate_ai_move(&game_state, &ai_move);
                
                if is_valid {
                    println!("AI requests move: {:?}", ai_move);
                    ev_make_move.send(MakeMoveEvent(ai_move));
                } else {
                    eprintln!("AI requested invalid move: {:?}, ignoring it", ai_move);
                    // Get a fallback move from legal moves if the AI's choice is invalid
                    if let Some(fallback_move) = get_fallback_move(&game_state) {
                        println!("Using fallback move instead: {:?}", fallback_move);
                        ev_make_move.send(MakeMoveEvent(fallback_move));
                    } else {
                        eprintln!("No valid moves available. Game might be in a terminal state.");
                    }
                }
            } else {
                eprintln!("AI task finished but returned no move. Game state might be terminal.");
            }
            commands.entity(entity).despawn();
            println!("Despawned AI task entity.");
            break;
        }
    }
}

/// Validate that an AI move is valid for the current game state
fn validate_ai_move(game_state: &GameState, ai_move: &Move) -> bool {
    // Check if the move is in the legal moves list
    let legal_moves = game_state.board.legal_moves();
    if !legal_moves.contains(ai_move) {
        println!("Move is not in legal moves list");
        return false;
    }
    
    // For normal moves, check that the source square has a piece of the correct color
    if let Some(from_square) = ai_move.from() {
        if let Some(piece) = game_state.board.board().piece_at(from_square) {
            if piece.color != game_state.current_player_turn {
                println!("Source square has piece of wrong color");
                return false;
            }
        } else {
            println!("Warning: Couldn't find piece at source square {:?}", from_square);
            return false;
        }
    }
    
    // Additional drawback-specific validation could be added here
    // For now, we trust the basic legality check from shakmaty
    
    true
}

/// Get a simple fallback move when the AI's chosen move is invalid
fn get_fallback_move(game_state: &GameState) -> Option<Move> {
    let legal_moves = game_state.board.legal_moves();
    if legal_moves.is_empty() {
        return None;
    }
    
    // Get the first legal move as a fallback
    // In a more sophisticated implementation, we might rank moves by simple heuristics
    let legal_moves_vec: Vec<Move> = legal_moves.into_iter().collect();
    legal_moves_vec.first().cloned()
} 