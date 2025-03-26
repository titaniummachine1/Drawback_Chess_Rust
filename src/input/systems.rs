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
    valid_moves: Query<Entity, With<ValidMoveDestination>>,
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
        let cursor_world_position = cursor_to_world_position(cursor_position, window, camera, camera_transform);
        
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
                break;
            }
        }
        
        if let Some(square) = clicked_square {
            // If there's a piece selected, check if this is a valid move
            if !selected.is_empty() {
                // Check if the click is on a valid move destination
                for entity in valid_moves.iter() {
                    if let Ok(valid_move) = valid_moves.get_component::<ValidMoveDestination>(entity) {
                        if valid_move.chess_move.to() == square {
                            // Valid move selected - send event to make the move
                            ev_make_move.send(MakeMoveEvent(valid_move.chess_move.clone()));
                            
                            // Clear selection and valid moves
                            for entity in selected.iter() {
                                commands.entity(entity).remove::<SelectedPiece>();
                            }
                            
                            for entity in valid_moves.iter() {
                                commands.entity(entity).despawn();
                            }
                            
                            return;
                        }
                    }
                }
            }
            
            // Check if there's a piece on the clicked square
            let mut clicked_on_piece = false;
            for (entity, piece, _) in pieces.iter() {
                if piece.pos == square && piece.color == game_state.current_player_turn {
                    // Clear previous selection and valid moves
                    for entity in selected.iter() {
                        commands.entity(entity).remove::<SelectedPiece>();
                    }
                    
                    for entity in valid_moves.iter() {
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
                                    (piece.pos.file().char() as u8 - b'a') as f32 - 3.5,
                                    (piece.pos.rank().char() as u8 - b'1') as f32 - 3.5,
                                    0.0,
                                ) * TILE_SIZE
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
                }
            }
            
            // If clicked on empty square or opponent's piece and nothing is selected, do nothing
            if !clicked_on_piece {
                // If clicking on empty square without a selected piece, clear selection
                for entity in selected.iter() {
                    commands.entity(entity).remove::<SelectedPiece>();
                }
                
                for entity in valid_moves.iter() {
                    commands.entity(entity).despawn();
                }
            }
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
    let window_size = Vec2::new(window.width(), window.height());
    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
    
    // Convert to world coordinates
    if let Some(matrix) = camera.viewport_to_world(camera_transform, ndc) {
        return matrix.origin.truncate();
    }
    
    Vec2::ZERO // Fallback
}

// Helper function to display valid moves for a selected piece
fn display_valid_moves(
    commands: &mut Commands,
    game_state: &GameState,
    from_square: Square,
    color: ChessColor,
    role: Role,
    board_squares: &Query<(&Transform, &BoardSquare)>,
) {
    let legals = game_state.board.legal_moves();
    
    // Filter moves to only those from the selected piece
    for chess_move in legals {
        if chess_move.from() == Some(from_square) {
            let to_square = chess_move.to();
            
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
                                transform.translation
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
}