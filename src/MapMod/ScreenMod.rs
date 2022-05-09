extern crate sdl2;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::{Rect, Point};

use super::{Tile, TileRenderer};
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
	pub fn draw(&self, tileRenderer: &mut TileRenderer, canvas: &mut Canvas<Window>) {
		let mut rect = Rect::new(0, 0, 50, 50);
		for tile in self.tiles.iter() {
			tileRenderer.draw(tile, canvas, rect);
			let (x, y) = (rect.top_left() + Point::from((50, 0))).into();
			let gtEq = !(self.width as i32 * 50 - x).is_positive();
			rect.reposition((x * (!gtEq) as i32, y + gtEq as i32 * 50));
		}
	}
}

