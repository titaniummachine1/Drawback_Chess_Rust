use bevy::prelude::*;
use shakmaty::{Square, Color as ChessColor, Role, Position, CastlingSide, EnPassantMode};
use crate::game_logic::state::GameState;
use crate::game_logic::systems::apply_move;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

// Maximum number of drawback IDs to hash (for index safety)
pub const MAX_DRAWBACK_INDICES: usize = 1024;

// Maximum number of RNG outcomes to pre-compute
pub const MAX_RNG_OUTCOMES: usize = 256;

#[derive(Resource, Clone, Debug)]
pub struct ZobristKeys {
    // Pieces[piece_type][square]
    pub pieces: [[u64; 64]; 12], // 12 piece types (6 piece types * 2 colors)
    
    // Side to move
    pub turn: u64,
    
    // Castling rights [white_kingside, white_queenside, black_kingside, black_queenside]
    pub castling: [u64; 4],
    
    // En passant file
    pub en_passant: [u64; 8],
    
    // Drawbacks - just need the drawback ID, not per color since we only care about current player
    pub drawbacks: [u64; MAX_DRAWBACK_INDICES],
    
    // RNG outcome
    pub rng_outcomes: [u64; MAX_RNG_OUTCOMES + 1] // +1 for "no outcome" state
}

pub struct ZobristPlugin;

impl Plugin for ZobristPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the Zobrist keys
        let zobrist_keys = initialize_zobrist_keys();
        app.insert_resource(zobrist_keys)
           .add_systems(Update, calculate_and_update_zobrist_hash.after(apply_move));
    }
}

// IMPORTANT: We use a fixed seed for initialization to ensure deterministic behavior
pub fn initialize_zobrist_keys() -> ZobristKeys {
    // Use a fixed seed for deterministic behavior - important for networked games
    // Fixed seed value: 42664 as a u64
    let seed = 42664;
    let mut rng = StdRng::seed_from_u64(seed);
    
    let mut keys = ZobristKeys {
        pieces: [[0; 64]; 12],
        turn: 0,
        castling: [0; 4],
        en_passant: [0; 8],
        drawbacks: [0; MAX_DRAWBACK_INDICES],
        rng_outcomes: [0; MAX_RNG_OUTCOMES + 1],
    };
    
    // Initialize piece keys
    for piece_type in 0..12 {
        for square in 0..64 {
            keys.pieces[piece_type][square] = rng.gen();
        }
    }
    
    // Initialize turn key
    keys.turn = rng.gen();
    
    // Initialize castling rights
    for i in 0..4 {
        keys.castling[i] = rng.gen();
    }
    
    // Initialize en passant file keys
    for i in 0..8 {
        keys.en_passant[i] = rng.gen();
    }
    
    // Initialize drawback keys - simplified to just per drawback ID
    for drawback_id in 0..MAX_DRAWBACK_INDICES {
        keys.drawbacks[drawback_id] = rng.gen();
    }
    
    // Initialize RNG outcome keys
    for i in 0..=MAX_RNG_OUTCOMES {
        keys.rng_outcomes[i] = rng.gen();
    }
    
    keys
}

/// Calculate and update the Zobrist hash in the GameState
pub fn calculate_and_update_zobrist_hash(
    mut game_state: ResMut<GameState>,
    zobrist_keys: Res<ZobristKeys>,
) {
    // Calculate the Zobrist hash
    let hash = calculate_zobrist_hash(&game_state, &zobrist_keys);
    
    // Update the game state with the new hash
    game_state.zobrist_hash = hash;
}

// Helper function to convert square to array index (0-63)
fn square_to_index(sq: Square) -> usize {
    let file_idx = sq.file().char() as usize - 'a' as usize;
    let rank_idx = sq.rank().char() as usize - '1' as usize;
    rank_idx * 8 + file_idx
}

// Helper function to convert piece type and color to array index
fn piece_to_index(role: Role, color: ChessColor) -> usize {
    let color_idx = match color {
        ChessColor::White => 0,
        ChessColor::Black => 6,
    };
    
    let role_idx = match role {
        Role::Pawn => 0,
        Role::Knight => 1,
        Role::Bishop => 2,
        Role::Rook => 3,
        Role::Queen => 4,
        Role::King => 5,
    };
    
    color_idx + role_idx
}

/// Calculate the Zobrist hash for a given GameState
pub fn calculate_zobrist_hash(game_state: &GameState, keys: &ZobristKeys) -> u64 {
    let mut hash: u64 = 0;
    let board = &game_state.board;
    let board_pieces = board.board();

    // 1. Pieces
    for square in Square::ALL {
        if let Some(piece) = board_pieces.piece_at(square) {
            let piece_idx = piece_to_index(piece.role, piece.color);
            let square_idx = square_to_index(square);
            hash ^= keys.pieces[piece_idx][square_idx];
        }
    }
    
    // 2. Side to move
    if game_state.current_player_turn == ChessColor::Black {
        hash ^= keys.turn;
    }
    
    // 3. Castling rights
    let castles = board.castles();
    
    if castles.has(ChessColor::White, CastlingSide::KingSide) {
        hash ^= keys.castling[0];
    }
    
    if castles.has(ChessColor::White, CastlingSide::QueenSide) {
        hash ^= keys.castling[1]; 
    }
    
    if castles.has(ChessColor::Black, CastlingSide::KingSide) {
        hash ^= keys.castling[2];
    }
    
    if castles.has(ChessColor::Black, CastlingSide::QueenSide) {
        hash ^= keys.castling[3];
    }
    
    // 4. En passant square
    if let Some(ep_square) = board.ep_square(EnPassantMode::Legal) {
        let file_idx = ep_square.file().char() as usize - 'a' as usize;
        hash ^= keys.en_passant[file_idx];
    }
    
    // 5. Only hash the current player's drawback - simplified approach
    let current_drawback = game_state.get_current_player_drawback_id();
    let drawback_idx = current_drawback.to_key_index() as usize % MAX_DRAWBACK_INDICES;
    hash ^= keys.drawbacks[drawback_idx];
    
    // 6. RNG outcome (if any)
    if let Some(outcome) = game_state.current_turn_rng_outcome {
        let outcome_idx = outcome as usize % (MAX_RNG_OUTCOMES + 1);
        hash ^= keys.rng_outcomes[outcome_idx];
    }
    
    hash
} 