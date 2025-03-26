use shakmaty::{Chess, Move, File};
use super::definition::DrawbackRule;
use super::registry::DrawbackId;

#[derive(Debug, Clone)]
pub struct BlockRandomFile;

impl DrawbackRule for BlockRandomFile {
    fn id(&self) -> DrawbackId { DrawbackId::BlockRandomFile }
    fn name(&self) -> &'static str { "Random File Blocked" }
    fn description(&self) -> &'static str { "At the start of your turn, a random file (A-H) is chosen. You cannot move any piece TO that file this turn." }

    fn needs_turn_rng(&self) -> bool {
        true // This rule requires per-turn RNG
    }

    fn get_rng_outcomes(&self) -> u8 {
        8 // 8 possible files (0-7 corresponding to A-H)
    }

    fn filter_pseudo_legal_moves(
        &self,
        _position: &Chess,
        moves: Vec<Move>,
        rng_outcome: Option<u8>, // Expecting 0-7 if RNG applies
    ) -> Vec<Move> {
        if let Some(blocked_file_index) = rng_outcome {
            if blocked_file_index < 8 {
                // Create a file from index (0-7 = a-h)
                let file_char = (b'a' + blocked_file_index) as char;
                let blocked_file = File::from_char(file_char).unwrap();
                
                println!("Applying BlockRandomFile: File '{}' is blocked this turn.", blocked_file);
                return moves.into_iter().filter(|mv| {
                    mv.to().file() != blocked_file
                }).collect();
            } else {
                eprintln!("BlockRandomFile: Invalid RNG outcome {}", blocked_file_index);
            }
        }
        
        // If RNG wasn't provided or was invalid, don't filter
        moves
    }

    fn check_loss_condition(&self, _position: &Chess, _legal_moves: &Vec<Move>) -> bool {
        false // No specific loss condition from this rule itself
    }
} 