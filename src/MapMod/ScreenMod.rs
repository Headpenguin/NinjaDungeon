use super::Tile;
use crate::Vec2d;

#[derive(Clone)]
pub struct Screen {
	width: u16,
	height: u16,
	tiles: Vec2d<Tile>,
	location: Location,
}

#[derive(Clone)]
pub struct Location {
	up: usize,
	down: usize,
	left: usize,
	right: usize,
}

impl Default for Location {
	fn default() -> Self {
		Location{
			up: 0,
			down: 0,
			left: 0,
			right:0,
		}
	}
}

impl Screen {
	pub fn new(width: u16, height: u16, location: Location) -> Screen {
		let mut v = vec![];
		v.resize(width as usize * height as usize, Tile::default());
		Screen {
			width,
			height,
			tiles: Vec2d::new(v, width as usize),
			location,
		}
	}
	pub fn draw(&self, canvas: &mut Canvas<Window>) {

	}
}

