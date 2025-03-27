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
use rand::Rng;

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

        // Debug output - show legal moves
        let legal_moves = game_state.board.legal_moves();
        println!("AI found {} legal moves on current board", legal_moves.len());
        
        // Print out board state for debugging
        println!("Current board state: {:?}", game_state.board);
        println!("Current player turn: {:?}", game_state.current_player_turn);
        
        // If no legal moves, we should handle that gracefully
        if legal_moves.is_empty() {
            println!("WARNING: No legal moves available for AI - should check if game is over");
            // We could handle this by transitioning to a different state
            return;
        }

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
    mut next_state: ResMut<NextState<TurnState>>,
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
                        // Change state to player's turn if AI can't move
                        next_state.set(TurnState::PlayerTurn);
                    }
                }
            } else {
                eprintln!("AI task finished but returned no move. Game state might be terminal.");
                
                // Check if there are actually no legal moves
                let legal_moves = game_state.board.legal_moves();
                if legal_moves.is_empty() {
                    eprintln!("No legal moves available. Setting state to player's turn.");
                    // Change state to player's turn if AI can't move
                    next_state.set(TurnState::PlayerTurn);
                } else if let Some(fallback_move) = get_fallback_move(&game_state) {
                    println!("Using fallback random move as AI couldn't decide: {:?}", fallback_move);
                    ev_make_move.send(MakeMoveEvent(fallback_move));
                }
            }
            commands.entity(entity).despawn();
            println!("Despawned AI task entity.");
            break;
        }
    }
}

/// Validate that an AI move is valid for the current game state
fn validate_ai_move(game_state: &GameState, proposed_move: &Move) -> bool {
    // Simply check if the move is in the current list of legal moves
    let legal_moves = game_state.board.legal_moves();
    legal_moves.contains(proposed_move)
}

/// Select a random fallback move from legal moves
fn get_fallback_move(game_state: &GameState) -> Option<Move> {
    let legal_moves = game_state.board.legal_moves();
    if legal_moves.is_empty() {
        return None;
    }
    
    let mut rng = rand::thread_rng();
    let random_idx = rng.gen_range(0..legal_moves.len());
    Some(legal_moves[random_idx].clone())
} 