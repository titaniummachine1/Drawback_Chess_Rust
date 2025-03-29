use bevy::prelude::*;
use crate::drawbacks::registry::DrawbackId;
use serde::{Serialize, Deserialize};

//==============================================================================
// GAME CONFIGURATION
// Edit the values below to change how the game works
//==============================================================================

// PLAYER CONFIGURATIONS
// ---------------------
// DEFAULT VALUES (Change these to configure your game)
//
// White Player (Bottom):
// - Set is_ai to true if you want AI to play as White, or false for human player
// - Set drawback to a number or name (see list below)
const WHITE_IS_AI: bool = false;
const WHITE_DRAWBACK_NAME: Option<&str> = None; // e.g. Some("No Castling")
const WHITE_DRAWBACK_INDEX: Option<u16> = None; // e.g. Some(1)

// Black Player (Top):
// - Set is_ai to true if you want AI to play as Black, or false for human player
// - Set drawback to a number or name (see list below)
const BLACK_IS_AI: bool = true;
const BLACK_DRAWBACK_NAME: Option<&str> = None;
const BLACK_DRAWBACK_INDEX: Option<u16> = None;

// AI SETTINGS
// -----------
// More iterations and deeper search = stronger but slower AI
const AI_ITERATION_LIMIT: u32 = 10000000; // Max iterations
const AI_TIME_LIMIT_MS: u32 = 3000;      // Always take 3 seconds
const AI_DEPTH_LIMIT: u8 = 24;           // Deep search
const AI_CHECK_QUIETNESS: bool = true;  
const AI_QUIESCENCE_DEPTH: u8 = 20;     

//==============================================================================
// DRAWBACK LIST
// ---------------------
// Names:
// - "No Castling"
// - "Pawns Advance One"
// - "Random File Blocked"
//
// Indices:
// - 1: No Castling
// - 2: Pawns Advance One
// - 3: Random File Blocked
//==============================================================================

/// Settings for an individual player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSettings {
    pub is_ai: bool,              // Whether this player is controlled by AI
    pub drawback: DrawbackSetting, // The drawback for this player
}

/// Settings for an individual drawback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawbackSetting {
    // Either name OR index should be specified (name takes precedence)
    pub name: Option<String>,    // Drawback name (e.g., "No Castling")
    pub index: Option<u16>,      // Drawback index (e.g., 1 for NoCastling)
}

/// AI algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSettings {
    pub iteration_limit: u32,     // Maximum MCTS iterations
    pub time_limit_ms: u32,       // Maximum time in milliseconds
    pub depth_limit: u8,          // Maximum search depth
    pub check_quietness: bool,    // Whether to check for quiet positions before ending search
    pub quiescence_depth: u8,     // Extra depth to search in non-quiet positions
}

/// Resource for storing game configuration
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    // Player settings
    pub white_player: PlayerSettings,
    pub black_player: PlayerSettings,
    
    // AI settings
    pub ai_settings: AiSettings,
}

impl Default for GameConfig {
    fn default() -> Self {
        // This function uses the constants from the top of the file
        Self {
            white_player: PlayerSettings {
                is_ai: WHITE_IS_AI,
                drawback: DrawbackSetting {
                    name: WHITE_DRAWBACK_NAME.map(|s| s.to_string()),
                    index: WHITE_DRAWBACK_INDEX,
                },
            },
            black_player: PlayerSettings {
                is_ai: BLACK_IS_AI,
                drawback: DrawbackSetting {
                    name: BLACK_DRAWBACK_NAME.map(|s| s.to_string()),
                    index: BLACK_DRAWBACK_INDEX,
                },
            },
            ai_settings: AiSettings {
                iteration_limit: AI_ITERATION_LIMIT,
                time_limit_ms: AI_TIME_LIMIT_MS,
                depth_limit: AI_DEPTH_LIMIT,
                check_quietness: AI_CHECK_QUIETNESS,
                quiescence_depth: AI_QUIESCENCE_DEPTH,
            },
        }
    }
}

// Predefined configurations
pub mod presets {
    use super::*;
    
    // Human vs AI (White: Human, Black: AI)
    pub fn human_vs_ai() -> GameConfig {
        GameConfig {
            white_player: PlayerSettings {
                is_ai: false, 
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            black_player: PlayerSettings {
                is_ai: true,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            ai_settings: AiSettings {
                iteration_limit: 1000000,
                time_limit_ms: 3000,
                depth_limit: 18,
                check_quietness: true,
                quiescence_depth: 16,
            },
        }
    }
    
    // AI vs Human (White: AI, Black: Human)
    pub fn ai_vs_human() -> GameConfig {
        GameConfig {
            white_player: PlayerSettings {
                is_ai: true,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            black_player: PlayerSettings {
                is_ai: false,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            ai_settings: AiSettings {
                iteration_limit: 1000000,
                time_limit_ms: 3000,
                depth_limit: 18,
                check_quietness: true,
                quiescence_depth: 16,
            },
        }
    }
    
    // Maximum AI Power (For best gameplay)
    pub fn max_power_ai() -> GameConfig {
        GameConfig {
            white_player: PlayerSettings {
                is_ai: true,  // Changed to true so both AIs can play against each other
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            black_player: PlayerSettings {
                is_ai: true,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            ai_settings: AiSettings {
                iteration_limit: 10000000,  // Very high iterations
                time_limit_ms: 3000,       // Fixed 3 second time
                depth_limit: 24,           // Deep search
                check_quietness: true,
                quiescence_depth: 20,
            },
        }
    }
    
    // Computer vs Computer (Both AI)
    pub fn ai_vs_ai() -> GameConfig {
        GameConfig {
            white_player: PlayerSettings {
                is_ai: true,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            black_player: PlayerSettings {
                is_ai: true,
                drawback: DrawbackSetting {
                    name: None, 
                    index: None,
                },
            },
            ai_settings: AiSettings {
                iteration_limit: 500000,
                time_limit_ms: 2000,
                depth_limit: 12,
                check_quietness: true,
                quiescence_depth: 8,
            },
        }
    }
    
    // Easy AI (Human vs Weak AI)
    pub fn easy_ai() -> GameConfig {
        GameConfig {
            white_player: PlayerSettings {
                is_ai: false,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            black_player: PlayerSettings {
                is_ai: true,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            ai_settings: AiSettings {
                iteration_limit: 200000,
                time_limit_ms: 1500,
                depth_limit: 8,
                check_quietness: false,
                quiescence_depth: 4,
            },
        }
    }
    
    // Strong AI (Human vs Strong AI)
    pub fn strong_ai() -> GameConfig {
        GameConfig {
            white_player: PlayerSettings {
                is_ai: false,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            black_player: PlayerSettings {
                is_ai: true,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            ai_settings: AiSettings {
                iteration_limit: 2000000,
                time_limit_ms: 5000,
                depth_limit: 24,
                check_quietness: true,
                quiescence_depth: 20,
            },
        }
    }
    
    // Smart AI - more focused on material evaluation
    pub fn smart_ai() -> GameConfig {
        GameConfig {
            white_player: PlayerSettings {
                is_ai: false,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            black_player: PlayerSettings {
                is_ai: true,
                drawback: DrawbackSetting {
                    name: None,
                    index: None,
                },
            },
            ai_settings: AiSettings {
                iteration_limit: 1500000,
                time_limit_ms: 3000,
                depth_limit: 20,
                check_quietness: true,
                quiescence_depth: 18,
            },
        }
    }
}

impl GameConfig {
    /// Resolve drawback ID from a setting
    pub fn resolve_drawback_id(&self, setting: &DrawbackSetting) -> DrawbackId {
        if setting.name.is_none() && setting.index.is_none() {
            return DrawbackId::None;
        }
        
        // Name takes precedence if both are specified
        if let Some(name) = &setting.name {
            match name.as_str() {
                "No Castling" => DrawbackId::NoCastling,
                "Pawns Advance One" => DrawbackId::PawnPushOneOnly,
                "Random File Blocked" => DrawbackId::BlockRandomFile,
                // Add more drawbacks here as they're implemented
                _ => {
                    eprintln!("Unknown drawback name: {}", name);
                    DrawbackId::None
                }
            }
        } else if let Some(index) = setting.index {
            match index {
                1 => DrawbackId::NoCastling,
                2 => DrawbackId::PawnPushOneOnly,
                3 => DrawbackId::BlockRandomFile,
                // Add more drawbacks here as they're implemented
                _ => {
                    eprintln!("Unknown drawback index: {}", index);
                    DrawbackId::None
                }
            }
        } else {
            DrawbackId::None
        }
    }
}

/// Plugin to handle game configuration
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        // Use smart_ai preset as default
        app.insert_resource(presets::smart_ai());
        
        // Comment out the default config line:
        // app.insert_resource(GameConfig::default());
        
        app.add_systems(Update, apply_config_to_game_state.run_if(resource_exists::<crate::game_logic::state::GameState>()));
    }
}

/// System to apply configuration to game state
fn apply_config_to_game_state(
    config: Res<GameConfig>,
    mut game_state: ResMut<crate::game_logic::state::GameState>,
) {
    // Only apply once
    static mut APPLIED: bool = false;
    unsafe {
        if APPLIED {
            return;
        }
        APPLIED = true;
    }

    // Set drawbacks based on configuration
    game_state.white_drawback = config.resolve_drawback_id(&config.white_player.drawback);
    game_state.black_drawback = config.resolve_drawback_id(&config.black_player.drawback);
    
    println!("Applied configuration:");
    println!("- White: AI={}, Drawback={:?}", 
             config.white_player.is_ai, game_state.white_drawback);
    println!("- Black: AI={}, Drawback={:?}", 
             config.black_player.is_ai, game_state.black_drawback);
    println!("- AI Settings: {}ms, depth={}, iterations={}", 
             config.ai_settings.time_limit_ms, 
             config.ai_settings.depth_limit,
             config.ai_settings.iteration_limit);
} 