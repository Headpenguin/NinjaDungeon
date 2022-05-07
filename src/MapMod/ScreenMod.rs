use super::Tile;
use crate::Vec2d;

pub struct Screen {
	width: u16,
	height: u16,
	tiles: Vec2d<Tile>,
	location: Location,
}

struct Location {
	up: usize,
	down: usize,
	left: usize,
	right: usize,
}

