use shakmaty::{Chess, Role, Square, Color, Piece, Position};
use std::collections::HashMap;

// Base piece values for midgame (mg) and endgame (eg)
pub const PIECE_VALUES: [(i32, i32); 6] = [
    (94, 100),    // Pawn
    (337, 281),   // Knight
    (365, 297),   // Bishop
    (479, 512),   // Rook
    (1025, 929),  // Queen
    (10000, 10000),   // King
];

// Phase weights for piece counting
const PIECE_PHASE_VALUES: [i32; 6] = [
    0,   // Pawn
    1,   // Knight
    1,   // Bishop
    2,   // Rook
    4,   // Queen
    0,   // King
];

// Maximum possible game phase score
const MAX_PHASE: f64 = 24.0;

// Piece-square tables - midgame for white perspective
// Pawns
const MG_PAWN_PST: [i32; 64] = [
    0,   0,   0,   0,   0,   0,   0,   0,
   50,  50,  50,  50,  50,  50,  50,  50,
   10,  10,  20,  35,  35,  20,  10,  10,
   10,  15,  30,  70,  70,  30,  15,  10,
    5,  10,  25,  55,  55,  25,  10,   5,
    5,   5,   5,   0,   0,   5,   5,   5,
    0,   0,   0, -30, -30,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0
];

// Knights
const MG_KNIGHT_PST: [i32; 64] = [
    -80, -50,  -30,  -30,  -30,  -30, -50, -80,
    -50, -20,    0,    0,    0,    0, -20, -50,
    -30,   0,   15,   20,   20,   15,   0, -30,
    -30,   5,   20,   25,   25,   20,   5, -30,
    -30,   0,   15,   20,   20,   15,   0, -30,
    -30,   5,   15,   15,   15,   15,   5, -30,
    -50, -20,    0,    5,    5,    0, -20, -50,
    -80, -50,  -30,  -30,  -30,  -30, -50, -80
];

// Bishops
const MG_BISHOP_PST: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,  15,   0,   0,   0,   0,  15, -10,
    -20, -10, -10, -10, -10, -10, -10, -20
];

// Rooks
const MG_ROOK_PST: [i32; 64] = [
     40, 40,  40,   0,   0,  40,  40, 40,
     5,  15,  15,  50,  50,  15,  50,  5,
     5,   0,   0,   0,   0,   0,   0,  5,
     5,   0,   0,   0,   0,   0,   0,  5,
     5,   0,   0,   0,   0,   0,   0,  5,
     5,   0,   0,   0,   0,   0,   0,  5,
     5,   0,   0,   0,   0,   0,   0,  5,
     0,  -5,   5,   5,   5,   10,  -5,  0
];

// Queens
const MG_QUEEN_PST: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,  -5,  -5,   0,   0, -10,
    -20, -10, -10,  -2,  -5, -10, -10, -20
];

// Kings
const MG_KING_PST: [i32; 64] = [
    -120, -120, -120, -120, -120, -120, -120, -120,
    -100, -100, -100, -100, -100, -100, -100, -100,
    -80, -80, -80, -80, -80, -80, -80, -80,
    -70, -70, -70, -70, -70, -70, -70, -70,
    -60, -60, -60, -60, -60, -60, -60, -60,
    -40, -40, -40, -40, -40, -40, -40, -40,
      0,   0, -10, -30, -30, -10,   0,   0,
     20,  50,  10,   0,   0,  10,  50,  20
];

// Piece-square tables - endgame for white perspective
// Pawns
const EG_PAWN_PST: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    400, 400, 400, 400, 400, 400, 400, 400,
     50,  55,  50,  50,  50,  50,  55,  50,
     30,  35,  30,  30,  30,  30,  35,  30,
     25,  20,  20,  20,  20,  20,  20,  25,
     15,  10,  10,  10,  10,  10,  10,  15,
     10,  10,  10,  10,  10,  10,  10,  10,
      0,   0,   0,   0,   0,   0,   0,   0
];

// Knights
const EG_KNIGHT_PST: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   0,   0,   0, -20, -40,
    -30,   0,  10,  15,  15,  10,   0, -30,
    -30,  10,  15,  20,  20,  15,  10, -30,
    -30,   0,  15,  20,  20,  15,   0, -30,
    -30,   5,  15,  15,  15,  15,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50
];

// Bishops
const EG_BISHOP_PST: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,  10,  10,   5,   0, -10,
    -10,   5,   5,  10,  10,   5,   5, -10,
    -10,   0,  10,  10,  10,  10,   0, -10,
    -10,  10,  10,  10,  10,  10,  10, -10,
    -10,  10,   0,   0,   0,   0,  10, -10,
    -20, -10, -10, -10, -10, -10, -10, -20
];

// Rooks
const EG_ROOK_PST: [i32; 64] = [
     40,  40,  40,   0,   0,  40,  40,  40,
      5,  10,  10,  10,  10,  10,  10,   5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
     -5,   0,   0,   0,   0,   0,   0,  -5,
      0,   0,  10,   5,   5,  10,   0,   0
];

// Queens
const EG_QUEEN_PST: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,   5,   5,   5,   0,  -5,
      0,   0,   5,   5,   5,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20
];

// Kings
const EG_KING_PST: [i32; 64] = [
    -50, -30, -30, -30, -30, -30, -30, -50,
    -30, -20, -20, -20, -20, -20, -20, -30,
    -30, -10,  -5,   0,   0, -5, -10, -30,
    -30, -10,   0,  10,  10,   0, -10, -30,
    -30, -10,   0,  10,  10,   0, -10, -30,
    -30, -10,  -5,   0,   0,  -5, -10, -30,
    -30, -20, -20, -20, -20, -20, -20, -30,
    -50, -30, -30, -30, -30, -30, -30, -50
];

// Lazy-initialized piece-square tables
// This will be pre-calculated once for both white and black
struct PieceSquareTables {
    white_mg: HashMap<Role, [i32; 64]>,
    white_eg: HashMap<Role, [i32; 64]>,
    black_mg: HashMap<Role, [i32; 64]>,
    black_eg: HashMap<Role, [i32; 64]>,
}

impl PieceSquareTables {
    fn new() -> Self {
        let mut white_mg = HashMap::new();
        let mut white_eg = HashMap::new();
        let mut black_mg = HashMap::new();
        let mut black_eg = HashMap::new();

        // Add white tables
        white_mg.insert(Role::Pawn, MG_PAWN_PST);
        white_mg.insert(Role::Knight, MG_KNIGHT_PST);
        white_mg.insert(Role::Bishop, MG_BISHOP_PST);
        white_mg.insert(Role::Rook, MG_ROOK_PST);
        white_mg.insert(Role::Queen, MG_QUEEN_PST);
        white_mg.insert(Role::King, MG_KING_PST);

        white_eg.insert(Role::Pawn, EG_PAWN_PST);
        white_eg.insert(Role::Knight, EG_KNIGHT_PST);
        white_eg.insert(Role::Bishop, EG_BISHOP_PST);
        white_eg.insert(Role::Rook, EG_ROOK_PST);
        white_eg.insert(Role::Queen, EG_QUEEN_PST);
        white_eg.insert(Role::King, EG_KING_PST);

        // Calculate black tables by mirroring and negating
        for role in [Role::Pawn, Role::Knight, Role::Bishop, Role::Rook, Role::Queen, Role::King] {
            let white_mg_table = white_mg.get(&role).unwrap();
            let white_eg_table = white_eg.get(&role).unwrap();
            
            let mut black_mg_table = [0; 64];
            let mut black_eg_table = [0; 64];
            
            for sq in 0..64 {
                let rank = sq / 8;
                let file = sq % 8;
                
                // Mirror square vertically
                let mirror_rank = 7 - rank;
                let mirror_sq = mirror_rank * 8 + file;
                
                // Negate the values for black
                black_mg_table[sq] = -white_mg_table[mirror_sq];
                black_eg_table[sq] = -white_eg_table[mirror_sq];
            }
            
            black_mg.insert(role, black_mg_table);
            black_eg.insert(role, black_eg_table);
        }

        Self {
            white_mg,
            white_eg,
            black_mg,
            black_eg,
        }
    }

    // Get the appropriate piece-square value
    fn get_piece_square_value(&self, piece: &Piece, sq: Square, is_endgame: f64) -> i32 {
        let sq_idx = Self::square_to_index(sq);
        let role = piece.role;
        let color = piece.color;

        // Get values from appropriate tables
        let mg_value = if color == Color::White {
            self.white_mg.get(&role).unwrap()[sq_idx]
        } else {
            self.black_mg.get(&role).unwrap()[sq_idx]
        };

        let eg_value = if color == Color::White {
            self.white_eg.get(&role).unwrap()[sq_idx]
        } else {
            self.black_eg.get(&role).unwrap()[sq_idx]
        };

        // Interpolate between midgame and endgame values
        // is_endgame is 0.0 for pure midgame, 1.0 for pure endgame
        let mg_phase = 1.0 - is_endgame;
        let eg_phase = is_endgame;
        
        ((mg_value as f64 * mg_phase) + (eg_value as f64 * eg_phase)) as i32
    }

    // Convert shakmaty Square to 0-63 index
    fn square_to_index(sq: Square) -> usize {
        let file = sq.file().char() as usize - 'a' as usize;
        let rank = sq.rank().char() as usize - '1' as usize;
        rank * 8 + file
    }
}

/// Calculate the game phase based on remaining pieces
pub fn compute_game_phase(board: &Chess) -> f64 {
    let mut phase = 0;
    
    for square in Square::ALL {
        if let Some(piece) = board.board().piece_at(square) {
            let role_idx = match piece.role {
                Role::Pawn => 0,
                Role::Knight => 1,
                Role::Bishop => 2,
                Role::Rook => 3,
                Role::Queen => 4,
                Role::King => 5,
            };
            
            phase += PIECE_PHASE_VALUES[role_idx];
        }
    }
    
    // Normalize to [0.0, 1.0] where 0.0 is midgame and 1.0 is endgame
    let phase = phase as f64;
    let phase = phase.min(MAX_PHASE) / MAX_PHASE;
    
    // Invert so 0 is midgame and 1 is endgame
    1.0 - phase
}

/// Evaluate a position using piece values and piece-square tables
pub fn evaluate_position_with_pst(board: &Chess) -> i32 {
    // Initialize piece-square tables (could be made static for performance)
    let pst = PieceSquareTables::new();
    
    // Determine game phase for interpolation
    let endgame_phase = compute_game_phase(board);
    println!("Game phase: {:.2} (0.0=midgame, 1.0=endgame)", endgame_phase);
    
    let mut score = 0;
    
    // Get the side to move
    let side_to_move = board.turn();
    
    // Evaluate pieces with position-dependent values
    for square in Square::ALL {
        if let Some(piece) = board.board().piece_at(square) {
            // Get role index for base piece value
            let role_idx = match piece.role {
                Role::Pawn => 0,
                Role::Knight => 1,
                Role::Bishop => 2,
                Role::Rook => 3,
                Role::Queen => 4,
                Role::King => 5,
            };
            
            // Calculate base piece value interpolated between midgame and endgame
            let (mg_value, eg_value) = PIECE_VALUES[role_idx];
            let piece_value = (mg_value as f64 * (1.0 - endgame_phase) + 
                              eg_value as f64 * endgame_phase) as i32;
            
            // Get position-dependent bonus from piece-square tables
            let position_value = pst.get_piece_square_value(&piece, square, endgame_phase);
            
            // Debug output for important pieces (uncomment for detailed debugging)
            if piece.role == Role::Queen || piece.role == Role::King {
                println!("{:?} {:?} at {:?} - material: {}, position: {}", 
                        piece.color, piece.role, square, piece_value, position_value);
            }
            
            // Add to score: positive for side to move, negative for opponent
            let value = piece_value + position_value;
            if piece.color == side_to_move {
                score += value;
            } else {
                score -= value;
            }
        }
    }
    
    score
} 