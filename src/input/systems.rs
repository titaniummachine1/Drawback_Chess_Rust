use bevy::prelude::*;
use crate::game_logic::events::MakeMoveEvent;
use crate::game_logic::state::{GameState, TurnState};
use crate::board::components::BoardSquare;
use crate::pieces::components::Piece;
use crate::constants::{SELECTED_COLOR, LEGAL_MOVE_COLOR, TILE_SIZE};
use shakmaty::{Move, Square, Role, Position, Color as ChessColor};

// Component to mark the currently selected piece
#[derive(Component)]
pub struct SelectedPiece;

// Component to mark squares as valid move destinations
#[derive(Component)]
pub struct ValidMoveDestination {
    pub chess_move: Move,
}

pub fn handle_piece_selection(
    mouse_button: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    board_squares: Query<(&Transform, &BoardSquare)>,
    pieces: Query<(Entity, &Piece, &Transform)>,
    mut commands: Commands,
    game_state: Res<GameState>,
    mut ev_make_move: EventWriter<MakeMoveEvent>,
    selected: Query<Entity, With<SelectedPiece>>,
    valid_moves: Query<(Entity, &ValidMoveDestination)>,
) {
    // Only process clicks when it's the player's turn
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    
    // Get the primary window and camera
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();
    
    // Get the cursor position in world coordinates
    if let Some(cursor_position) = window.cursor_position() {
        println!("Mouse clicked at screen position: {:?}", cursor_position);
        let cursor_world_position = cursor_to_world_position(cursor_position, window, camera, camera_transform);
        
        // Find the closest board square to the click
        let closest_square = find_closest_board_square(cursor_world_position, &board_squares);
        
        if let Some((pos, square)) = closest_square {
            println!("Clicked closest to square: {:?} at position {:?}", square, pos);
            
            // First check if there's a piece on the clicked square
            println!("Looking for piece at square: {:?}", square);
            let mut clicked_on_piece = false;
            let mut clicked_on_selected_piece = false;
            
            // Check if we already have a selected piece
            if !selected.is_empty() {
                // Get the currently selected piece
                for (entity, piece, _) in pieces.iter() {
                    if selected.contains(entity) {
                        // If we're clicking on the same piece that's already selected, deselect it
                        if piece.pos == square {
                            println!("Deselecting currently selected piece");
                            clicked_on_selected_piece = true;
                            
                            // Clear selection and valid moves
                            for entity in selected.iter() {
                                commands.entity(entity).remove::<SelectedPiece>();
                            }
                            
                            for (entity, _) in valid_moves.iter() {
                                commands.entity(entity).despawn();
                            }
                        }
                        break;
                    }
                }
                
                // If we're not clicking on the selected piece, check if we're clicking on a valid move
                if !clicked_on_selected_piece {
                    // Check if the click is on a valid move destination
                    for (_entity, valid_move) in valid_moves.iter() {
                        if valid_move.chess_move.to() == square {
                            // Valid move selected - send event to make the move
                            println!("Making move: {:?}", valid_move.chess_move);
                            ev_make_move.send(MakeMoveEvent(valid_move.chess_move.clone()));
                            
                            // Also print additional debug info
                            println!(">>> MAKING MOVE: from {:?} to {:?} <<<", 
                                     valid_move.chess_move.from(), 
                                     valid_move.chess_move.to());
                            
                            // Clear selection and valid moves
                            for entity in selected.iter() {
                                commands.entity(entity).remove::<SelectedPiece>();
                            }
                            
                            for (entity, _) in valid_moves.iter() {
                                commands.entity(entity).despawn();
                            }
                            
                            return;
                        }
                    }
                }
            }
            
            // If we get here, either we didn't click on the selected piece or a valid move,
            // so check if we clicked on another piece
            if !clicked_on_selected_piece {
                for (entity, piece, _) in pieces.iter() {
                    if piece.pos == square {
                        println!("Found piece matching square: {:?} {:?}", piece.color, piece.role);
                        if piece.color == game_state.current_player_turn {
                            println!("Selected piece: {:?} {:?} at {:?}", piece.color, piece.role, piece.pos);
                            
                            // Clear previous selection and valid moves
                            for entity in selected.iter() {
                                commands.entity(entity).remove::<SelectedPiece>();
                            }
                            
                            for (entity, _) in valid_moves.iter() {
                                commands.entity(entity).despawn();
                            }
                            
                            // Mark this piece as selected
                            commands.entity(entity).insert(SelectedPiece);
                            clicked_on_piece = true;
                            
                            // Spawn a highlight sprite for the selected piece
                            commands.spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: SELECTED_COLOR,
                                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                                        ..default()
                                    },
                                    transform: Transform::from_translation(
                                        Vec3::new(
                                            ((piece.pos.file().char() as u8 - b'a') as f32 - 3.5) * TILE_SIZE,
                                            ((7 - (piece.pos.rank().char() as u8 - b'1')) as f32 - 3.5) * TILE_SIZE,
                                            -0.1, // Place slightly below pieces for correct layering
                                        )
                                    ),
                                    ..default()
                                },
                                SelectedPiece, // Tag with same component so it gets cleaned up
                            ));
                            
                            // Find and display valid moves for this piece
                            display_valid_moves(
                                &mut commands, 
                                &game_state,
                                piece.pos, 
                                piece.color, 
                                piece.role,
                                &board_squares,
                            );
                            
                            break;
                        } else {
                            println!("Piece belongs to opponent, not selectable");
                        }
                    }
                }
            }
            
            // If we didn't click on a friendly piece, clear selection (unless we already did above)
            if !clicked_on_piece && !clicked_on_selected_piece {
                println!("No friendly piece at square or clicked on empty square");
                // If clicking on empty square without a selected piece, clear selection
                for entity in selected.iter() {
                    commands.entity(entity).remove::<SelectedPiece>();
                }
                
                for (entity, _) in valid_moves.iter() {
                    commands.entity(entity).despawn();
                }
            }
        } else {
            println!("No board squares found to click on");
        }
    }
}

// Helper function to convert cursor position to world coordinates
fn cursor_to_world_position(
    cursor_pos: Vec2,
    _window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Vec2 {
    // For 2D games with a Camera2dBundle, use viewport_to_world_2d
    // This correctly handles the camera's view and projection
    match camera.viewport_to_world_2d(camera_transform, cursor_pos) {
        Some(world_pos) => {
            println!("Converted screen ({:?}) to world ({:?})", cursor_pos, world_pos);
            world_pos
        }
        None => {
            println!("Failed to convert cursor position to world coordinates");
            Vec2::ZERO
        }
    }
}

// Helper function to find the closest board square to a world position
fn find_closest_board_square(
    world_pos: Vec2,
    board_squares: &Query<(&Transform, &BoardSquare)>,
) -> Option<(Vec2, Square)> {
    let mut closest_square = None;
    let mut closest_distance = f32::MAX;
    
    for (transform, board_square) in board_squares.iter() {
        let square_pos = transform.translation.truncate();
        let distance = world_pos.distance_squared(square_pos);
        
        println!("Square: {:?} at position {:?}, distance: {}", board_square.square, square_pos, distance);
        
        if distance < closest_distance {
            closest_distance = distance;
            closest_square = Some((square_pos, board_square.square));
        }
    }
    
    closest_square
}

// Helper function to display valid moves for a selected piece
fn display_valid_moves(
    commands: &mut Commands,
    game_state: &GameState,
    from_square: Square,
    _color: ChessColor,
    _role: Role,
    board_squares: &Query<(&Transform, &BoardSquare)>,
) {
    let legals = game_state.board.legal_moves();
    println!("Found {} legal moves in total", legals.len());
    
    // Filter moves to only those from the selected piece
    let mut valid_move_count = 0;
    for chess_move in legals {
        if chess_move.from() == Some(from_square) {
            valid_move_count += 1;
            let to_square = chess_move.to();
            println!("Valid move: {:?} to {:?}", from_square, to_square);
            
            // Find the board square entity for the destination
            for (transform, board_square) in board_squares.iter() {
                if board_square.square == to_square {
                    // Create a visual indicator for the valid move
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: LEGAL_MOVE_COLOR,
                                custom_size: Some(Vec2::new(TILE_SIZE / 2.0, TILE_SIZE / 2.0)), // Make larger and more visible
                                ..default()
                            },
                            transform: Transform::from_translation(
                                Vec3::new(
                                    transform.translation.x,
                                    transform.translation.y,
                                    -0.05, // Place slightly below pieces but above selection highlight
                                )
                            ),
                            ..default()
                        },
                        ValidMoveDestination { chess_move },
                    ));
                    
                    break;
                }
            }
        }
    }
    println!("Displaying {} valid moves for selected piece", valid_move_count);
}