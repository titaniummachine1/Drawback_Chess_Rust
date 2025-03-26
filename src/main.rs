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
// The images directory contains assets, not Rust code, so no need to import it as a module

// Use module plugins
use game_logic::plugin::GameLogicPlugin;
use board::plugin::BoardPlugin;
use pieces::plugin::PiecesPlugin;
use input::plugin::InputPlugin;
use ai::{AiPlugin, ZobristPlugin}; // Import Zobrist from AI
use ui::plugin::UiPlugin;
use drawbacks::DrawbacksPlugin; // Use the drawbacks plugin (registers rules)

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
             primary_window: Some(Window {
                 title: "Drawback Chess".into(),
                 // Make window exactly match board size for better coordinate translation
                 resolution: (constants::BOARD_SIZE_PX, constants::BOARD_SIZE_PX).into(),
                 resizable: false,
                 position: WindowPosition::Centered(MonitorSelection::Primary),
                 ..default()
             }),
             ..default()
        }))
        // --- Plugin Ordering ---
        // 1. Register Drawback Rules First (Needed by GameState/Logic)
        .add_plugins(DrawbacksPlugin)
        // 2. Core Logic (incl. GameState, Events)
        .add_plugins(GameLogicPlugin) // Initializes GameState, schedules apply_move
        // 3. Zobrist Hashing (after GameState)
        .add_plugins(ZobristPlugin) // Initialize Zobrist keys from AI module
        // 4. Visual Setup
        .add_plugins(BoardPlugin)
        .add_plugins(PiecesPlugin) // Spawns pieces based on initial GameState
        // 5. UI Setup
        .add_plugins(UiPlugin)
        // 6. Input Handling (Needs board/pieces/GameState)
        .add_plugins(InputPlugin)
        // 7. AI Logic (Needs GameState, DrawbackRegistry)
        .add_plugins(AiPlugin)
        .run();
} 