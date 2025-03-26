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
    mouse_button: Res<ButtonInput<MouseButton>>,
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
        println!("World position: {:?}", cursor_world_position);
        
        // Print all board square positions for debugging
        println!("Board squares positions:");
        for (transform, board_square) in board_squares.iter().take(5) { // Show just a few for clarity
            println!("Square {:?} at position: {:?}", board_square.square, transform.translation.truncate());
        }
        
        // Check if a square was clicked
        let mut clicked_square: Option<Square> = None;
        for (transform, board_square) in board_squares.iter() {
            let square_pos = transform.translation.truncate();
            let half_size = TILE_SIZE / 2.0;
            
            // Check if cursor is within this square's bounds
            if cursor_world_position.x >= square_pos.x - half_size
                && cursor_world_position.x <= square_pos.x + half_size
                && cursor_world_position.y >= square_pos.y - half_size
                && cursor_world_position.y <= square_pos.y + half_size {
                clicked_square = Some(board_square.square);
                println!("Clicked on square: {:?} at position {:?}", board_square.square, square_pos);
                break;
            }
        }
        
        if let Some(square) = clicked_square {
            // If there's a piece selected, check if this is a valid move
            if !selected.is_empty() {
                // Check if the click is on a valid move destination
                for (entity, valid_move) in valid_moves.iter() {
                    if valid_move.chess_move.to() == square {
                        // Valid move selected - send event to make the move
                        println!("Making move: {:?}", valid_move.chess_move);
                        ev_make_move.send(MakeMoveEvent(valid_move.chess_move.clone()));
                        
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
            
            // Check if there's a piece on the clicked square
            println!("Looking for piece at square: {:?}", square);
            let mut clicked_on_piece = false;
            for (entity, piece, _) in pieces.iter() {
                println!("Checking piece: {:?} {:?} at {:?}", piece.color, piece.role, piece.pos);
                if piece.pos == square {
                    println!("Found piece matching square");
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
            
            // If clicked on empty square or opponent's piece and nothing is selected, do nothing
            if !clicked_on_piece {
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
            println!("Click didn't land on any chess board square");
        }
    }
}

// Helper function to convert cursor position to world coordinates
fn cursor_to_world_position(
    cursor_pos: Vec2,
    window: &Window,
    camera: &Camera,
    camera_transform: &GlobalTransform,
) -> Vec2 {
    // Convert cursor position to normalized device coordinates (NDC)
    let ndc = (cursor_pos / Vec2::new(window.width(), window.height())) * 2.0 - Vec2::ONE;
    
    // Use viewport_to_world to convert NDC to world coordinates
    if let Some(ray) = camera.viewport_to_world(camera_transform, ndc) {
        // For a 2D game, we're only interested in the XY coordinates
        ray.origin.truncate()
    } else {
        println!("Failed to convert cursor position to world coordinates");
        Vec2::ZERO
    }
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
                                custom_size: Some(Vec2::new(TILE_SIZE / 3.0, TILE_SIZE / 3.0)),
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