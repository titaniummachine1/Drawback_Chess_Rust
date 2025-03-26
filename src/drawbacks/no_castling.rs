use shakmaty::{Chess, Move};
use super::definition::DrawbackRule;
use super::registry::DrawbackId; // Use the ID enum

#[derive(Debug, Clone)]
pub struct NoCastling;

impl DrawbackRule for NoCastling {
    fn id(&self) -> DrawbackId { DrawbackId::NoCastling } // Return Enum ID
    fn name(&self) -> &'static str { "No Castling" }
    fn description(&self) -> &'static str { "Castling (Kingside or Queenside) is not allowed." }

    fn filter_pseudo_legal_moves(
        &self,
         _position: &Chess,
         moves: Vec<Move>,
         _rng_outcome: Option<u8>, // Ignored
    ) -> Vec<Move> {
        moves.into_iter().filter(|mv| !matches!(mv, Move::Castle { .. })).collect()
    }

    fn check_loss_condition(&self, _position: &Chess, _legal_moves: &Vec<Move>) -> bool {
        false
    }
} 