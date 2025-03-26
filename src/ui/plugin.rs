use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui);
    }
}

fn setup_ui(mut commands: Commands) {
    println!("Setting up UI...");
    
    // Add a camera with a clear view of the board
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0.0, 0.0, 999.9),
        ..default()
    });
    
    // Setup UI elements - to be implemented based on game requirements
} 