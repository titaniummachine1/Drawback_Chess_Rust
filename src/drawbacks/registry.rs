use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use super::definition::DrawbackRule;
// Import the actual drawback structs:
use super::no_castling::NoCastling;
use super::pawn_push_one::PawnPushOneOnly;
use super::block_random_file::BlockRandomFile;

/// Enum of all available drawbacks.
/// This enum provides a way to:
/// 1. Uniquely identify a drawback rule
/// 2. Enable zero-cost switching in match statements vs. using strings
/// 3. Converting to/from u8/u16 for storage and Zobrist hashing
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum DrawbackId {
    None, // Special case for no drawback
    NoCastling,
    PawnPushOneOnly,
    BlockRandomFile,
    // ... Add all other drawback IDs here ...
    // Example: CannotCaptureKnights,
    // Example: KingMustMoveForward,
}

/// Methods for DrawbackId enum
impl DrawbackId {
    /// Get a numeric index for Zobrist hashing
    /// Must be unique across all drawbacks
    pub fn to_key_index(&self) -> u16 {
        match self {
            Self::None => 0,
            Self::NoCastling => 1,
            Self::PawnPushOneOnly => 2,
            Self::BlockRandomFile => 3,
            // ... Map others to sequential IDs ...
        }
    }
}

/// Resource mapping DrawbackId enum values to actual implementations.
/// Provides lookup capabilities for the game to find rule implementations.
#[derive(Resource)]
pub struct DrawbackRegistry {
    pub rules: HashMap<DrawbackId, Arc<dyn DrawbackRule + Send + Sync>>,
}

impl Default for DrawbackRegistry {
    fn default() -> Self {
        initialize_drawback_registry()
    }
}

/// Plugin responsible for setting up the DrawbackRegistry resource.
pub struct DrawbacksPlugin;

impl Plugin for DrawbacksPlugin {
    fn build(&self, app: &mut App) {
        let registry = initialize_drawback_registry();
        app.insert_resource(registry);
        println!("DrawbackRegistry initialized with rules.");
    }
}

/// Initializes the registry by creating instances of all known drawbacks.
/// In a real app with 200+, this might load from configs or use macros.
fn initialize_drawback_registry() -> DrawbackRegistry {
    let mut rules = HashMap::new();

    // --- Instantiate and insert ALL available drawbacks ---
    let no_castling_rule = Arc::new(NoCastling) as Arc<dyn DrawbackRule + Send + Sync>;
    rules.insert(no_castling_rule.id(), no_castling_rule);

    let pawn_push_one_rule = Arc::new(PawnPushOneOnly) as Arc<dyn DrawbackRule + Send + Sync>;
    rules.insert(pawn_push_one_rule.id(), pawn_push_one_rule);

    let block_random_file_rule = Arc::new(BlockRandomFile) as Arc<dyn DrawbackRule + Send + Sync>;
    rules.insert(block_random_file_rule.id(), block_random_file_rule);

    // ... Add ALL other ~200 rule instances here ...

    println!("Loading drawbacks into registry...");
    DrawbackRegistry { rules }
} 