use bevy::prelude::*;
use bevy::tasks::Task;
use shakmaty::Move;

/// Component for AI thinking task
#[derive(Component)]
pub struct AiThinking(pub Task<Option<Move>>); 