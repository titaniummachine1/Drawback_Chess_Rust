use shakmaty::{Chess, Move, Role};
use super::definition::DrawbackRule;
use super::registry::DrawbackId; // Use the ID enum

#[derive(Debug, Clone)]
pub struct PawnPushOneOnly;

impl DrawbackRule for PawnPushOneOnly {
    fn id(&self) -> DrawbackId { DrawbackId::PawnPushOneOnly } // Return Enum ID
    fn name(&self) -> &'static str { "Pawns Advance One" }
    fn description(&self) -> &'static str { "Pawns may not advance two squares on their first move." }

    fn filter_pseudo_legal_moves(
        &self,
        _position: &Chess,
        moves: Vec<Move>,
        _rng_outcome: Option<u8>, // Ignored
    ) -> Vec<Move> {
         moves.into_iter().filter(|mv| {
             match mv {
                Move::Normal { role, from, to, .. } => {
                    if *role == Role::Pawn {
                         // Calculate absolute difference between ranks
                         let from_rank = from.rank().char() as i8 - '1' as i8;
                         let to_rank = to.rank().char() as i8 - '1' as i8;
                         let rank_diff = (to_rank - from_rank).abs();
                         
                         rank_diff != 2 // Allow only if NOT a double push
                    } else {
                        true // Allow non-pawn moves
                    }
                }
                 _ => true, // Allow castling, en passant (these might be filtered by other rules later)
             }
         }).collect()
    }

    fn check_loss_condition(&self, _position: &Chess, _legal_moves: &Vec<Move>) -> bool {
        false
    }
} 