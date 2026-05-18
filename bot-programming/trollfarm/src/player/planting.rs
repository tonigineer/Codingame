use crate::entities::TreeType;
use crate::game::Game;
use crate::position::Position;

use std::collections::HashMap;

use super::Player;

impl Player {
    fn planting(_game: &Game, _player: &Player) -> Option<HashMap<TreeType, Position>> {
        // TODO: implement planting strategy
        None
    }
}
