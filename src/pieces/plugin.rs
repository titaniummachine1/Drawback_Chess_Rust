use bevy::prelude::*;
use shakmaty::{Color as ChessColor, Role, Square, Position, Move};
use crate::constants::TILE_SIZE;
use crate::game_logic::state::{GameState, TurnState};
use crate::game_logic::events::MakeMoveEvent;
use super::components::Piece;

// Component for promotion UI
#[derive(Component)]
struct PromotionUI;

#[derive(Component)]
struct PromotionOption {
    role: Role,
    from: Square,
    to: Square,
    color: ChessColor,
}

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pieces)
           .add_systems(Update, update_piece_positions)
           .add_systems(Update, handle_promotion_selection);
    }
}

/// Handle clicks on promotion piece options
fn handle_promotion_selection(
    mut commands: Commands,
    query: Query<(Entity, &PromotionOption, &Parent)>,
    ui_query: Query<Entity, With<PromotionUI>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut ev_make_move: EventWriter<MakeMoveEvent>,
) {
    // Only process clicks
    if !mouse_buttons.just_pressed(MouseButton::Left) {
        return;
    }
    
    // Get cursor position
    let window = windows.single();
    let (camera, camera_transform) = camera_q.single();
    
    if let Some(cursor_pos) = window.cursor_position() {
        // Convert screen position to world coordinates
        let world_pos = screen_to_world(cursor_pos, window, camera, camera_transform);
        println!("Mouse clicked at screen position: {:?}", cursor_pos);
        println!("Converted screen ({:?}) to world ({:?})", cursor_pos, world_pos);
        
        // Check if we clicked on a promotion option
        for (entity, option, parent) in query.iter() {
            // Calculate the expected position of each promotion UI option (match logic in spawn_promotion_ui)
            let file = option.to.file().char() as u8 - b'a';
            let rank = option.to.rank().char() as u8 - b'1';
            let promotion_roles = [Role::Queen, Role::Rook, Role::Bishop, Role::Knight];
            
            // Find the index of this role in the array to determine its position
            if let Some(role_index) = promotion_roles.iter().position(|r| *r == option.role) {
                let offset = (role_index as f32 - 1.5) * 1.2; // Match offset in spawn_promotion_ui
                let option_pos = Vec2::new(
                    ((file as f32 - 3.5) * TILE_SIZE) + offset * TILE_SIZE,
                    ((7 - rank) as f32 - 3.5) * TILE_SIZE
                );
                
                // Check if clicked within range of this option (increased radius for easier clicking)
                let distance = (option_pos - world_pos).length_squared();
                if distance < (TILE_SIZE * 0.8).powi(2) { // Increased detection radius
                    println!("Selected promotion: {:?}", option.role);
                    
                    // Create the promotion move
                    let promotion_move = Move::Normal {
                        role: Role::Pawn,
                        from: option.from,
                        capture: None, // We'll let the game logic handle capture detection
                        to: option.to,
                        promotion: Some(option.role),
                    };
                    
                    // Log the promotion move details for debugging
                    println!("Creating promotion move: from={:?}, to={:?}, role={:?}", 
                             option.from, option.to, option.role);
                    
                    // Send the move event
                    ev_make_move.send(MakeMoveEvent(promotion_move));
                    
                    // Remove all promotion UI
                    for ui_entity in ui_query.iter() {
                        commands.entity(ui_entity).despawn_recursive();
                    }
                    
                    break;
                }
            }
        }
    }
}

// Helper function to convert screen to world coordinates
fn screen_to_world(
    cursor_pos: Vec2,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Vec2 {
    // Get the size of the window
    let window_size = Vec2::new(window.width(), window.height());
    
    // Convert screen position [0..resolution] to ndc [-1..1]
    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
    
    // Convert to world space
    let world_pos = camera_transform.compute_matrix() 
                  * camera.projection_matrix().inverse()
                  * Vec4::new(ndc.x, ndc.y, 0.0, 1.0);
    
    Vec2::new(world_pos.x, world_pos.y)
}

/// Update piece positions when moves are made
pub fn update_piece_positions(
    mut commands: Commands,
    mut pieces: Query<(Entity, &mut Piece, &mut Transform)>,
    mut ev_make_move: EventReader<MakeMoveEvent>,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
    current_state: Res<State<TurnState>>,
) {
    for ev in ev_make_move.read() {
        let chess_move = &ev.0;
        println!("Updating piece positions for move: {:?}", chess_move);
        
        // Handle different types of moves
        match chess_move {
            // Handle normal moves and captures
            shakmaty::Move::Normal { from, to, promotion, .. } => {
                // Check if this is a pawn reaching the last rank without a promotion specified
                if promotion.is_none() {
                    if let Some(piece) = game_state.board.board().piece_at(*from) {
                        if piece.role == Role::Pawn {
                            // Check if pawn is reaching the last rank
                            let is_white_to_eighth = piece.color == ChessColor::White && to.rank().char() == '8';
                            let is_black_to_first = piece.color == ChessColor::Black && to.rank().char() == '1';
                            
                            if (is_white_to_eighth || is_black_to_first) && 
                               *current_state.get() == TurnState::PlayerTurn {
                                // Show promotion UI and don't process the move yet
                                spawn_promotion_ui(&mut commands, &asset_server, *from, *to, piece.color);
                                continue;
                            }
                        }
                    }
                }
                
                // First, remove any captured pieces at the destination
                for (entity, piece, _) in pieces.iter() {
                    if piece.pos == *to {
                        println!("Removing captured piece at {:?}", to);
                        commands.entity(entity).despawn();
                    }
                }
                
                // Then find and update the piece we're moving
                let mut found = false;
                for (entity, mut piece, mut transform) in pieces.iter_mut() {
                    if piece.pos == *from {
                        // Update the piece's square
                        println!("Moving piece from {:?} to {:?}", from, to);
                        piece.pos = *to;
                        
                        // Handle promotion
                        if let Some(promotion_role) = promotion {
                            piece.role = *promotion_role;
                            
                            // Update the sprite
                            let color_prefix = match piece.color {
                                ChessColor::White => "w",
                                ChessColor::Black => "b",
                            };
                            
                            let role_suffix = match *promotion_role {
                                Role::Queen => "Q",
                                Role::Rook => "R",
                                Role::Bishop => "B", 
                                Role::Knight => "N",
                                _ => "Q", // Default to queen (shouldn't happen)
                            };
                            
                            let image_path = format!("images/{}{}.png", color_prefix, role_suffix);
                            // We need to update the texture, but this requires more complex handling
                            // For now, we'll despawn and respawn the piece
                            commands.entity(entity).despawn();
                            spawn_single_piece(&mut commands, &asset_server, *to, piece.color, *promotion_role);
                            found = true;
                            break;
                        }
                        
                        // Update its visual position
                        let file = to.file().char() as u8 - b'a';
                        let rank = to.rank().char() as u8 - b'1';
                        
                        let new_position = Vec3::new(
                            (file as f32 - 3.5) * TILE_SIZE,
                            ((7 - rank) as f32 - 3.5) * TILE_SIZE,
                            0.1
                        );
                        
                        transform.translation = new_position;
                        found = true;
                        break;
                    }
                }
                
                if !found {
                    println!("Warning: Couldn't find piece at source square {:?}", from);
                }
            },
            
            // Handle en passant
            shakmaty::Move::EnPassant { from, to } => {
                // Remove the captured pawn (which is not at the destination square)
                let captured_square = Square::from_coords(to.file(), from.rank());
                for (entity, piece, _) in pieces.iter() {
                    if piece.pos == captured_square {
                        println!("Removing en passant captured piece at {:?}", captured_square);
                        commands.entity(entity).despawn();
                    }
                }
                
                // Move the pawn
                for (_, mut piece, mut transform) in pieces.iter_mut() {
                    if piece.pos == *from {
                        piece.pos = *to;
                        
                        let file = to.file().char() as u8 - b'a';
                        let rank = to.rank().char() as u8 - b'1';
                        
                        let new_position = Vec3::new(
                            (file as f32 - 3.5) * TILE_SIZE,
                            ((7 - rank) as f32 - 3.5) * TILE_SIZE,
                            0.1
                        );
                        
                        transform.translation = new_position;
                        break;
                    }
                }
            },
            
            // Handle castling
            shakmaty::Move::Castle { king, rook } => {
                // Calculate new positions for king and rook
                let king_to = if *rook > *king { 
                    Square::from_coords(rook.file().offset(-1).unwrap(), king.rank()) // King moves right
                } else {
                    Square::from_coords(rook.file().offset(1).unwrap(), king.rank()) // King moves left
                };
                
                let rook_to = if *rook > *king {
                    Square::from_coords(king.file().offset(1).unwrap(), king.rank()) // Rook moves left
                } else {
                    Square::from_coords(king.file().offset(-1).unwrap(), king.rank()) // Rook moves right
                };
                
                // Move the king
                for (_, mut piece, mut transform) in pieces.iter_mut() {
                    if piece.pos == *king && piece.role == Role::King {
                        piece.pos = king_to;
                        
                        let file = king_to.file().char() as u8 - b'a';
                        let rank = king_to.rank().char() as u8 - b'1';
                        
                        let new_position = Vec3::new(
                            (file as f32 - 3.5) * TILE_SIZE,
                            ((7 - rank) as f32 - 3.5) * TILE_SIZE,
                            0.1
                        );
                        
                        transform.translation = new_position;
                        break;
                    }
                }
                
                // Move the rook
                for (_, mut piece, mut transform) in pieces.iter_mut() {
                    if piece.pos == *rook && piece.role == Role::Rook {
                        piece.pos = rook_to;
                        
                        let file = rook_to.file().char() as u8 - b'a';
                        let rank = rook_to.rank().char() as u8 - b'1';
                        
                        let new_position = Vec3::new(
                            (file as f32 - 3.5) * TILE_SIZE,
                            ((7 - rank) as f32 - 3.5) * TILE_SIZE,
                            0.1
                        );
                        
                        transform.translation = new_position;
                        break;
                    }
                }
            },
            
            // Don't need to handle Drop or Put - not part of standard chess
            _ => {
                println!("Special move type not fully implemented: {:?}", chess_move);
            }
        }
    }
}

// Function to spawn promotion UI
fn spawn_promotion_ui(
    commands: &mut Commands,
    asset_server: &AssetServer,
    from: Square,
    to: Square,
    color: ChessColor,
) {
    // Calculate position for the promotion UI
    let file = to.file().char() as u8 - b'a';
    let rank = to.rank().char() as u8 - b'1';
    
    let base_position = Vec3::new(
        (file as f32 - 3.5) * TILE_SIZE,
        ((7 - rank) as f32 - 3.5) * TILE_SIZE,
        0.5 // Above pieces (increased z-value for better visibility)
    );
    
    // Spawn parent entity for all promotion options
    let parent = commands.spawn((
        SpatialBundle::default(),
        PromotionUI,
    )).id();
    
    // Spawn the four promotion options
    let promotion_roles = [Role::Queen, Role::Rook, Role::Bishop, Role::Knight];
    let color_prefix = match color {
        ChessColor::White => "w",
        ChessColor::Black => "b",
    };
    
    // Display options horizontally instead of vertically for better visibility
    for (i, role) in promotion_roles.iter().enumerate() {
        // Position horizontally with offset based on file position
        let offset = (i as f32 - 1.5) * 1.2; // Spread out the options more
        let position = Vec3::new(
            base_position.x + offset * TILE_SIZE,
            base_position.y,
            base_position.z
        );
        
        let role_suffix = match role {
            Role::Queen => "Q",
            Role::Rook => "R",
            Role::Bishop => "B",
            Role::Knight => "N",
            _ => "Q", // Default (shouldn't happen)
        };
        
        let image_path = format!("images/{}{}.png", color_prefix, role_suffix);
        
        // Spawn the promotion option with a highlighted background for better visibility
        commands.entity(parent).with_children(|parent| {
            // First spawn a background highlight
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgba(1.0, 1.0, 0.0, 0.5), // Yellow semi-transparent
                    custom_size: Some(Vec2::new(TILE_SIZE * 1.1, TILE_SIZE * 1.1)),
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            });
            
            // Then spawn the piece image
            parent.spawn((
                SpriteBundle {
                    texture: asset_server.load(&image_path),
                    transform: Transform::from_translation(position),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE_SIZE * 1.0, TILE_SIZE * 1.0)),
                        ..default()
                    },
                    ..default()
                },
                PromotionOption {
                    role: *role,
                    from,
                    to,
                    color,
                },
            ));
        });
    }
    
    println!("Spawned promotion UI at {:?}", to);
}

/// Spawns chess pieces based on the current game state
pub fn spawn_pieces(
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
) {
    println!("Spawning chess pieces...");
    
    // Iterate through all squares on the board
    for square in Square::ALL {
        if let Some(piece) = game_state.board.board().piece_at(square) {
            // Calculate position based on square
            // File is horizontal (A-H, left to right)
            let file = square.file().char() as u8 - b'a'; // 0-7 for files a-h
            
            // Rank is vertical (1-8, bottom to top)
            // In chess, rank 1 is at the bottom (white's first rank)
            // and rank 8 is at the top (black's first rank)
            let rank = square.rank().char() as u8 - b'1'; // 0-7 for ranks 1-8
            
            // Debug information about coordinate mapping
            println!(
                "Mapping square {:?}: file={}, rank={}, file_char={}, rank_char={}", 
                square, 
                file, 
                rank, 
                square.file().char(),
                square.rank().char()
            );
            
            // In our screen coordinates:
            // - x increases from left to right (matches chess files)
            // - y increases from bottom to top (matches chess ranks)
            // - (0,0) is the center of the screen
            let position = Vec3::new(
                (file as f32 - 3.5) * TILE_SIZE, // Center the board horizontally
                ((7 - rank) as f32 - 3.5) * TILE_SIZE, // Flip the rank to get 0 at the bottom
                0.1, // Place slightly above board and highlights
            );

            println!("Placing piece at square: {:?}, position: {:?}", square, position);

            // Determine piece image path based on color and role
            let color_prefix = match piece.color {
                ChessColor::White => "w",
                ChessColor::Black => "b",
            };
            
            let role_suffix = match piece.role {
                Role::King => "K",
                Role::Queen => "Q",
                Role::Rook => "R",
                Role::Bishop => "B",
                Role::Knight => "N",
                Role::Pawn => "P",
            };
            
            // Load the piece image
            let image_path = format!("images/{}{}.png", color_prefix, role_suffix);
            println!("Loading piece image: {}", image_path);
            
            // Spawn piece with image
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(&image_path),
                    transform: Transform::from_translation(position),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.9)),
                        ..default()
                    },
                    ..default()
                },
                Piece {
                    pos: square,
                    color: piece.color,
                    role: piece.role,
                },
            ));
        }
    }
}

// Helper function to spawn a single piece
fn spawn_single_piece(
    commands: &mut Commands,
    asset_server: &AssetServer,
    square: Square,
    color: ChessColor,
    role: Role,
) {
    // Calculate position based on square
    let file = square.file().char() as u8 - b'a';
    let rank = square.rank().char() as u8 - b'1';
    
    let position = Vec3::new(
        (file as f32 - 3.5) * TILE_SIZE,
        ((7 - rank) as f32 - 3.5) * TILE_SIZE,
        0.1
    );
    
    // Determine piece image path
    let color_prefix = match color {
        ChessColor::White => "w",
        ChessColor::Black => "b",
    };
    
    let role_suffix = match role {
        Role::King => "K",
        Role::Queen => "Q",
        Role::Rook => "R",
        Role::Bishop => "B",
        Role::Knight => "N",
        Role::Pawn => "P",
    };
    
    // Load the piece image
    let image_path = format!("images/{}{}.png", color_prefix, role_suffix);
    
    // Spawn piece with image
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load(&image_path),
            transform: Transform::from_translation(position),
            sprite: Sprite {
                custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.9)),
                ..default()
            },
            ..default()
        },
        Piece {
            pos: square,
            color,
            role,
        },
    ));
} 