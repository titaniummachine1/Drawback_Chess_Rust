use bevy::prelude::*;
use std::ops::Add;
use crate::game_logic::events::MakeMoveEvent;
use crate::game_logic::state::{GameState, TurnState};
use crate::board::components::BoardSquare;
use crate::pieces::components::Piece;
use crate::constants::{SELECTED_COLOR, LEGAL_MOVE_COLOR, TILE_SIZE, Z_LEGAL_MOVES, Z_HIGHLIGHT, Z_PIECES};
use shakmaty::{Move, Square, Role, Position, Color as ChessColor};

// Component to mark the currently selected piece
#[derive(Component)]
pub struct SelectedPiece;

// Component to mark squares as valid move destinations
#[derive(Component)]
pub struct ValidMoveDestination {
    pub chess_move: Move,
}

// Add a new component to differentiate selection highlights from move indicators
#[derive(Component)]
pub struct PieceSelectionHighlight;

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
    selection_highlights: Query<Entity, With<PieceSelectionHighlight>>,
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
        
        // Find the closest board square to the click
        let closest_square = find_closest_board_square(cursor_world_position, &board_squares);
        
        if let Some((_, square)) = closest_square {
            println!("Clicked on square: {:?}", square);
            
            // First, check if clicked on a valid move destination
            let mut clicked_on_valid_move = false;
            
            for (_, valid_move) in valid_moves.iter() {
                if valid_move.chess_move.to() == square {
                    // Valid move selected - send event to make the move
                    println!("Making move: {:?}", valid_move.chess_move);
                    ev_make_move.send(MakeMoveEvent(valid_move.chess_move.clone()));
                    clicked_on_valid_move = true;
                    
                    // Clear selection and valid moves
                    clear_selection(&mut commands, &selected, &valid_moves, &selection_highlights);
                    return;
                }
            }
            
            // If we didn't click on a valid move, then we're either:
            // 1. Clicking on a piece to select it
            // 2. Clicking on an empty square or opponent's piece (deselect)
            // 3. Clicking on the currently selected piece (deselect)
            
            // Always clear the current selection first
            clear_selection(&mut commands, &selected, &valid_moves, &selection_highlights);
            
            // Now check if we're clicking on a friendly piece to select it
            let mut found_friendly_piece = false;
            
            for (entity, piece, _) in pieces.iter() {
                if piece.pos == square && piece.color == game_state.current_player_turn {
                    println!("Selected piece: {:?} {:?} at {:?}", piece.color, piece.role, piece.pos);
                    
                    // Mark this piece as selected
                    commands.entity(entity).insert(SelectedPiece);
                    found_friendly_piece = true;
                    
                    // Calculate the piece's file and rank
                    let file = piece.pos.file().char() as u8 - b'a';
                    let rank = piece.pos.rank().char() as u8 - b'1';
                    
                    // Calculate highlight position based on board orientation
                    let highlight_pos = calculate_highlight_position(
                        file as usize,
                        rank as usize,
                        Z_HIGHLIGHT,
                        game_state.board_flipped
                    );
                    
                    // Spawn a highlight sprite for the selected piece
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: SELECTED_COLOR,
                                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                                ..default()
                            },
                            transform: Transform::from_translation(highlight_pos),
                            ..default()
                        },
                        PieceSelectionHighlight, // Use the new component for selection highlights
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
            
            if !found_friendly_piece {
                println!("No friendly piece at square or clicked on empty square");
                // Selection was already cleared above
            }
        } else {
            println!("No board square found under click");
            // Click is outside the board, clear selection
            clear_selection(&mut commands, &selected, &valid_moves, &selection_highlights);
        }
    }
}

// Helper function to clear current selection
fn clear_selection(
    commands: &mut Commands,
    selected: &Query<Entity, With<SelectedPiece>>,
    valid_moves: &Query<(Entity, &ValidMoveDestination)>,
    selection_highlights: &Query<Entity, With<PieceSelectionHighlight>>,
) {
    // Remove the SelectedPiece component from actual pieces
    for entity in selected.iter() {
        if !valid_moves.iter().any(|(e, _)| e == entity) {
            commands.entity(entity).remove::<SelectedPiece>();
        }
    }
    
    // Despawn all selection highlights
    for entity in selection_highlights.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    // Despawn all valid move indicators
    for (entity, _) in valid_moves.iter() {
        commands.entity(entity).despawn_recursive();
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

// Add a helper function to calculate visual positions based on board orientation
fn calculate_highlight_position(file: usize, rank: usize, z: f32, board_flipped: bool) -> Vec3 {
    if board_flipped {
        // Flipped board (white at top)
        Vec3::new(
            (file as f32 - 3.5) * TILE_SIZE,
            (rank as f32 - 3.5) * TILE_SIZE,
            z
        )
    } else {
        // Standard board (white at bottom)
        Vec3::new(
            (file as f32 - 3.5) * TILE_SIZE,
            ((7 - rank) as f32 - 3.5) * TILE_SIZE,
            z
        )
    }
}

// Helper function to display valid moves for a selected piece
fn display_valid_moves(
    commands: &mut Commands,
    game_state: &GameState,
    from_square: Square,
    piece_color: ChessColor,
    piece_role: Role,
    board_squares: &Query<(&Transform, &BoardSquare)>,
) {
    // Get all legal moves for the current game state
    let legals = game_state.board.legal_moves();
    println!("Found {} total legal moves", legals.len());
    
    // Debug output of all legal moves
    for m in &legals {
        if let Some(sq) = m.from() {
            println!("Legal move from: {:?} to {:?}", sq, m.to());
        } else {
            println!("Special move (no source square) to {:?}", m.to());
        }
    }
    
    // Filter moves to only those from the selected piece's square
    let mut valid_move_count = 0;
    for chess_move in legals {
        // Check if this move starts from our selected piece's square
        if let Some(from) = chess_move.from() {
            if from == from_square {
                valid_move_count += 1;
                let to_square = chess_move.to();
                println!("Valid move: {:?} to {:?} for {:?} {:?}", 
                         from_square, to_square, piece_color, piece_role);
                
                // Find the board square entity for the destination
                for (_, board_square) in board_squares.iter() {
                    if board_square.square == to_square {
                        // Determine if this is a capture move
                        let is_capture = match game_state.board.board().piece_at(to_square) {
                            Some(_) => true,
                            None => false
                        };
                        
                        // Calculate file and rank for the destination square
                        let file = to_square.file().char() as u8 - b'a';
                        let rank = to_square.rank().char() as u8 - b'1';
                        
                        // Set up position for the indicator based on board orientation
                        let position = calculate_highlight_position(
                            file as usize, 
                            rank as usize, 
                            Z_LEGAL_MOVES,
                            game_state.board_flipped
                        );
                        
                        if is_capture {
                            // For captures, create a red circle indicator (capture)
                            commands.spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::rgba(0.9, 0.2, 0.2, 0.7), // Bright red for captures
                                        custom_size: Some(Vec2::new(TILE_SIZE * 0.4, TILE_SIZE * 0.4)),
                                        ..default()
                                    },
                                    transform: Transform::from_translation(position),
                                    ..default()
                                },
                                ValidMoveDestination { chess_move: chess_move.clone() },
                            ));
                        } else {
                            // For regular moves, create a smaller green circle
                            commands.spawn((
                                SpriteBundle {
                                    sprite: Sprite {
                                        color: Color::rgba(0.2, 0.8, 0.2, 0.7), // Bright green for regular moves
                                        custom_size: Some(Vec2::new(TILE_SIZE * 0.3, TILE_SIZE * 0.3)),
                                        ..default()
                                    },
                                    transform: Transform::from_translation(position),
                                    ..default()
                                },
                                ValidMoveDestination { chess_move },
                            ));
                        }
                        
                        break;
                    }
                }
            }
        }
    }
    
    println!("Displaying {} valid moves for selected piece at {:?}", valid_move_count, from_square);
    if valid_move_count == 0 {
        println!("WARNING: No valid moves found for selected piece at {:?}", from_square);
    }
}