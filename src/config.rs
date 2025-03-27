use bevy::prelude::*;
use std::fs;
use std::path::Path;
use crate::drawbacks::registry::DrawbackId;
use serde::{Serialize, Deserialize};

/// Resource for storing game configuration
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    // Player drawback settings
    pub white_drawback: DrawbackSetting,
    pub black_drawback: DrawbackSetting,
    
    // AI settings
    pub ai_settings: AiSettings,
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

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            white_drawback: DrawbackSetting {
                name: None,
                index: None,
            },
            black_drawback: DrawbackSetting {
                name: None,
                index: None,
            },
            ai_settings: AiSettings {
                iteration_limit: 10000,
                time_limit_ms: 3000,
                depth_limit: 14,
                check_quietness: true,
                quiescence_depth: 6,
            },
        }
    }
}

impl GameConfig {
    /// Load configuration from a file
    pub fn load_from_file(path: &str) -> Self {
        if Path::new(path).exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(config) => {
                            println!("Loaded configuration from {}", path);
                            return config;
                        },
                        Err(e) => {
                            eprintln!("Error parsing config file: {}", e);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error reading config file: {}", e);
                }
            }
        } else {
            println!("Config file not found, using defaults");
        }
        
        // Return default if loading fails
        let default_config = Self::default();
        
        // Save the default config for future reference
        if let Err(e) = default_config.save_to_file(path) {
            eprintln!("Error saving default config: {}", e);
        }
        
        default_config
    }
    
    /// Save configuration to a file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        println!("Configuration saved to {}", path);
        Ok(())
    }
    
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
        // Load configuration from file
        let config = GameConfig::load_from_file("drawback_chess_config.json");
        
        // Add configuration as a resource
        app.insert_resource(config);
        
        // Add systems - but we'll apply config to game state in Update systems
        // to ensure GameState is already created by the GameLogicPlugin
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
    game_state.white_drawback = config.resolve_drawback_id(&config.white_drawback);
    game_state.black_drawback = config.resolve_drawback_id(&config.black_drawback);
    
    println!("Applied drawbacks from config - White: {:?}, Black: {:?}", 
             game_state.white_drawback, game_state.black_drawback);
} 