use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
    }
}

fn setup_ui(mut commands: Commands) {
    println!("Setting up UI...");
    
    // Add a camera
    commands.spawn(Camera2dBundle::default());
    
    // Setup UI elements - to be implemented based on game requirements
} 