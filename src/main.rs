use bevy::prelude::*;

// --- Modules ---
mod constants;
mod game_logic;
mod board;
mod pieces;
mod input;
mod ai;
mod ui;
mod drawbacks; // Import the drawbacks module
mod config; // Import the configuration module
// The images directory contains assets, not Rust code, so no need to import it as a module

// Use module plugins
use game_logic::plugin::GameLogicPlugin;
use board::plugin::BoardPlugin;
use pieces::plugin::PiecesPlugin;
use input::plugin::InputPlugin;
use ai::{AiPlugin, ZobristPlugin}; // Import Zobrist from AI
use ui::plugin::UiPlugin;
use drawbacks::DrawbacksPlugin; // Use the drawbacks plugin (registers rules)
use config::ConfigPlugin; // Use the config plugin

fn main() {
    // Make sure the window is large enough to show the entire board
    let window_width = constants::BOARD_SIZE_PX;
    let window_height = constants::BOARD_SIZE_PX;

    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1))) // Dark background
        .add_plugins(DefaultPlugins.set(WindowPlugin {
             primary_window: Some(Window {
                 title: "Drawback Chess".into(),
                 // Make window large enough to show the entire board
                 resolution: (window_width, window_height).into(),
                 resizable: false,
                 position: WindowPosition::Centered(MonitorSelection::Primary),
                 // Make sure the window has correct scaling
                 present_mode: bevy::window::PresentMode::AutoVsync,
                 ..default()
             }),
             ..default()
        }))
        // --- Plugin Ordering ---
        // 1. Configuration Plugin (needs to be first to properly set up config)
        .add_plugins(ConfigPlugin)
        // 2. Register Drawback Rules (Needed by GameState/Logic)
        .add_plugins(DrawbacksPlugin)
        // 3. Core Logic (incl. GameState, Events)
        .add_plugins(GameLogicPlugin) // Initializes GameState, schedules apply_move
        // 4. Zobrist Hashing (after GameState)
        .add_plugins(ZobristPlugin) // Initialize Zobrist keys from AI module
        // 5. Visual Setup
        .add_plugins(BoardPlugin)
        .add_plugins(PiecesPlugin) // Spawns pieces based on initial GameState
        // 6. UI Setup
        .add_plugins(UiPlugin)
        // 7. Input Handling (Needs board/pieces/GameState)
        .add_plugins(InputPlugin)
        // 8. AI Logic (Needs GameState, DrawbackRegistry)
        .add_plugins(AiPlugin)
        .run();
} 