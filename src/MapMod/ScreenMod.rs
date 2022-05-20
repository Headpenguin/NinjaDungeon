use std::ptr::addr_of_mut;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::{Rect, Point};

use BinaryFileIO::BinaryDataContainer::SelfContained;
use BinaryFileIO::BFStream::{ProvideReferencesDynamic, DynamicBinaryTranslator, ProvidePointersMutDynamic, DynamicTypedTranslator};

use super::{Tile, TileRenderer, Map, CollisionType};
use crate::{Vec2d, Direction, Vector};

const TILE_DIVISOR: f32 = 1f32/50f32;

#[derive(Clone)]
pub struct Screen {
	width: u16,
	height: u16,
	tiles: Vec2d<Tile>,
	position: (u32, u32),
}

#[derive(Clone, Copy)]
pub struct Location {
	up: usize,
	down: usize,
	left: usize,
	right: usize,
}

pub struct CollisionBounds {
	startX: u16,
	endX: u16,
	endY: u16,
	x: u16,
	y: u16,
}

impl Location {
	pub fn new(up: usize, down: usize, left: usize, right: usize) -> Location {
		Location{up, down, left, right}
	}
}

unsafe impl SelfContained for Location {}

impl Iterator for CollisionBounds {
	type Item = (u16, u16);
	fn next(&mut self) -> Option<Self::Item> {
		let result = Some((self.x, self.y));
		if self.y > self.endY {
			return None;
		}
		if self.x >= self.endX {
			self.x = self.startX;
			self.y += 1;
		}
		else{self.x += 1;}
		result
	}
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
	pub fn new(width: u16, height: u16, position: (u32, u32)) -> Screen {
		let mut v = vec![];
		v.resize(width as usize * height as usize, Tile::default());
		Screen {
			width,
			height,
			tiles: Vec2d::new(v, width as usize),
			position,
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
	pub fn replaceTile(&mut self, position: (u16, u16), replacement: Tile) {
		*self.tiles.indexMut(position.1 as usize, position.0 as usize) = replacement;
	}
	pub fn getTile(&self, position: (u16, u16)) -> &Tile {
		self.tiles.index(position.1 as usize, position.0 as usize)
	}
    pub fn getDimensions(&self) -> (u16, u16) {(self.width, self.height)}
	fn pointToIndex(&self, point: Point) -> (u16, u16) {	
		let x = (point.x() as f32 * TILE_DIVISOR).floor().clamp(0f32, (self.width - 1) as f32) as u16;
        let y = (point.y() as f32 * TILE_DIVISOR).floor().clamp(0f32, (self.height - 1) as f32) as u16;
		(x, y)
	}
	pub fn calculateCollisionBounds(&self, hitbox: Rect) -> CollisionBounds {
		let (startX, y) = self.pointToIndex(hitbox.top_left());
		let (endX, endY) = self.pointToIndex(hitbox.bottom_right());
		CollisionBounds	{
			x: startX,
			startX,
			endX,
			endY,
			y,
		}
	}
	pub fn collide<'a>(&'a self, bounds: &mut CollisionBounds) -> Option<((u16, u16), &'a Tile)> {
		let location = bounds.next()?;
		Some((location, self.getTile(location)))
	}
	fn getPosition(&self, topLeft: (u32, u32), center: Point, direction: Direction) -> Point {
		let (x, y) = topLeft;
		let v = Vector::fromPoints((self.position.0 as f32, self.position.1 as f32), (x as f32, y as f32));
		match direction {
			Direction::Up => {Point::new(center.x() + v.0 as i32, self.height as i32 * 50)},
			Direction::Down => {Point::new(center.x() + v.0 as i32, 0)},
			Direction::Left => {Point::new(self.width as i32 * 50, center.y() + v.1 as i32)},
			Direction::Right => {Point::new(0, center.y() + v.1 as i32)},
		}
	}
    pub fn getScreen(&self, center: Point, map: &Map) -> Option<(usize, Point)> {
		let tile = self.getTile(self.pointToIndex(center));
		if let CollisionType::Transition(screen) = tile.getCollisionType() {
			let direction = if center.x() < 0 {Direction::Left}
			else if center.x() >= self.width as i32 * 50 {Direction::Right}
			else if center.y() < 0 {Direction::Up}
			else {Direction::Down};
			Some((screen, map.getScreen(screen).getPosition(self.position, center, direction)))
		}
		else {None}
    }
}

impl<'a> ProvideReferencesDynamic<'a> for Screen {
	type Type = Self;
	fn provideReferencesDyn<T: DynamicBinaryTranslator<'a>>(&'a self, translator: &mut T) {
		self.tiles.provideReferencesDyn(translator);
	}
}

impl<'a> ProvidePointersMutDynamic<'a> for Screen {
	type Type = Self;
	unsafe fn providePointersMutDyn<T: DynamicTypedTranslator<'a>>(uninitialized: *mut Self, depth: usize, translator: &mut T) -> bool {
		Vec2d::providePointersMutDyn(addr_of_mut!((*uninitialized).tiles), depth, translator)
	}
}


