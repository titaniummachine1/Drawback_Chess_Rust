// Publicly export core definitions and the plugin
pub mod definition;
pub mod registry;
pub mod no_castling;
pub mod pawn_push_one;
pub mod block_random_file;

pub use registry::{DrawbackRegistry, DrawbackId, DrawbacksPlugin};

// Modules for specific drawback implementations
// ... include modules for all other drawbacks ... 