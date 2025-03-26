use shakmaty::{Chess, Move};
use std::fmt::Debug;
 // Use Arc for sharing
use super::registry::DrawbackId; // Use the new ID type

/// Trait defining the interface for a Drawback rule.
/// Must be `Send + Sync` and implement `Debug`.
pub trait DrawbackRule: Send + Sync + Debug {
    /// Returns the unique Enum ID for this drawback.
    fn id(&self) -> DrawbackId;

    /// Gets a human-readable name.
    fn name(&self) -> &'static str;

    /// Gets a description of the rule.
    fn description(&self) -> &'static str;

    /// Indicates if this drawback requires a random value determined at the START of the owning player's turn.
    /// For example, a rule blocking a random file needs this to be true.
    fn needs_turn_rng(&self) -> bool {
        false // Default to false
    }

    /// If `needs_turn_rng` is true, this indicates the number of possible discrete outcomes (e.g., 8 for files).
    /// The outcome will be passed as `Option<u8>` from 0 to N-1.
    fn get_rng_outcomes(&self) -> u8 {
         1 // Default, ignored if needs_turn_rng is false
    }

    /// Takes a list of pseudo-legal moves and filters them according to this rule.
    /// `position`: The current board state *before* the move.
    /// `moves`: The list of moves generated so far (possibly filtered by other means).
    /// `rng_outcome`: The result of the per-turn RNG (0 to N-1), if `needs_turn_rng` was true for this rule.
    /// It should NOT check for leaving the king in check unless that is part of the rule itself.
    fn filter_pseudo_legal_moves(
        &self,
        position: &Chess,
        moves: Vec<Move>,
        rng_outcome: Option<u8>, // Added RNG outcome parameter
    ) -> Vec<Move>;

    /// Checks if a specific loss condition imposed by this drawback is met.
    /// `position`: The state AFTER the opponent's last move (it's the current player's turn).
    /// `legal_moves`: The list of moves available to the current player *after all filtering*.
    /// Returns `true` if the current player loses due to this rule.
    fn check_loss_condition(&self, position: &Chess, legal_moves: &Vec<Move>) -> bool;

    // Potential future methods...
} 