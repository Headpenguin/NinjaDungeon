use std::collections::HashSet;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::{Rect, Point};

//use BinaryFileIO::BinaryDataContainer::SelfContained;
//use BinaryFileIO::BFStream::{ProvideReferencesDynamic, DynamicBinaryTranslator, ProvidePointersMutDynamic, DynamicTypedTranslator};

use serde::{Serialize, Deserialize};

use super::{Tile, TileRenderer, InnerMap, CollisionType};
use super::TileMod;
use crate::{Vec2d, Direction, Vector, ID};
use crate::IntHasher::UInt64Hasher;

const TILE_DIVISOR: f32 = 1f32/50f32;

#[derive(Serialize, Deserialize, Clone)]
pub struct Screen {
	width: u16,
	height: u16,
	tiles: Vec2d<Tile>,
	entities: HashSet<u64, UInt64Hasher>,
	position: (u32, u32),
}

#[derive(Serialize, Deserialize, Clone, Copy)]
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

//unsafe impl SelfContained for Location {}

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
		//println!("{}", self.y);
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
			entities: HashSet::default(),
		}
	}
	pub fn containsPoint(&self, point: Point) -> bool {
		Rect::new(self.position.0 as i32, self.position.1 as i32, self.width as u32, self.height as u32).contains_point(point)
	}
	pub fn draw(&self, tileRenderer: &mut TileRenderer, canvas: &mut Canvas<Window>, topLeft: Point) {
		let mut rect = Rect::new(-topLeft.x, -topLeft.y, 50, 50);
		for tile in self.tiles.iter() {
			tileRenderer.draw(tile, canvas, rect);
			let (x, y) = (rect.top_left() + Point::from((50, 0))).into();
			let gtEq = !(self.width as i32 * 50 - topLeft.x - x).is_positive();
			rect.reposition((x * (!gtEq) as i32 - topLeft.x * gtEq as i32, y + gtEq as i32 * 50));
		}
	}
	pub fn generateIconRect(&self, scaleX: f32, scaleY: f32, topLeft: Point) -> Rect {
		Rect::new(((self.position.0 as f32 - topLeft.x as f32) * scaleX) as i32, 
			((self.position.1 as f32 - topLeft.y as f32) * scaleY) as i32, 
			(self.width as f32 * scaleX) as u32, 
			(self.height as f32 * scaleY) as u32)
	}
	pub fn addEntity(&mut self, id: ID) -> bool {
		self.entities.insert(id.getID())
	}
	pub fn removeEntity(&mut self, id: ID) -> bool {
		self.entities.remove(&id.getID())
	}
	pub fn getEntitiesIter<'a>(&'a self) -> impl Iterator<Item=ID> + 'a {
		self.entities.iter().map(|id| ID::new(*id, 0))
	}
	pub fn iconDraw(&self, tileRenderer: &mut TileRenderer, canvas: &mut Canvas<Window>, location: Rect) {
		let (xIncrement, yIncrement) = (location.width() as f32 / self.width as f32, location.height() as f32 / self.height as f32);
		let (mut posX, mut posY) = (location.x() as f32, location.y() as f32);
		let mut rect = Rect::new(location.left(), location.top(), xIncrement as u32, yIncrement as u32);
		for tile in self.tiles.iter() {
			tileRenderer.draw(tile, canvas, rect);
			posX += xIncrement;
			let gtEq = (location.right() as f32 - posX - 0.001).is_sign_negative();
			posY += gtEq as u8 as f32 * yIncrement;
			posX = posX * (!gtEq) as u8 as f32 + gtEq as u8 as f32 * location.x() as f32;
			rect.reposition((posX as i32, posY as i32));
		}
	}
	pub fn replaceTile(&mut self, position: (u16, u16), replacement: Tile) {
		*self.tiles.indexMut(position.1 as usize, position.0 as usize) = replacement;
	}
	pub fn getTile(&self, position: (u16, u16)) -> &Tile {
		self.tiles.get(position.1 as usize, position.0 as usize).unwrap_or(&TileMod::OOB)
	}
	pub fn moveToPosition(&mut self, position: (u32, u32)) {
		self.position = position;
	}
    pub fn getDimensions(&self) -> (u16, u16) {(self.width, self.height)}
	pub fn getMaxScreenCoords(&self) -> (u32, u32) {(self.width as u32 * 50, self.height as u32 * 50)}
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
		let v = Vector::fromPoints((self.position.0 as f32, self.position.1 as f32), (x as f32, y as f32)) * 50f32;
		match direction {
			Direction::Up => {Point::new(center.x() + v.0 as i32, self.height as i32 * 50 - 3)},
			Direction::Down => {Point::new(center.x() + v.0 as i32, 0)},
			Direction::Left => {Point::new(self.width as i32 * 50 - 3, center.y() + v.1 as i32)},
			Direction::Right => {Point::new(0, center.y() + v.1 as i32)},
		}
	}
    pub fn getScreen(&self, center: Point, map: &InnerMap) -> Option<(usize, Point)> {
		let tile = self.getTile(self.pointToIndex(center));
		if let CollisionType::Transition(screen) = tile.getCollisionType() {
			let direction = if center.x() < 0 {Direction::Left}
			else if center.x() >= self.width as i32 * 50 {Direction::Right}
			else if center.y() < 0 {Direction::Up}
			else {Direction::Down};
			if let Some(result) = map.getScreen(screen) {
				Some((screen, result.getPosition(self.position, center, direction)))
			}
			else {None}
		}
		else {None}
    }
}

impl Default for Screen {
	fn default() -> Self {
		Screen {
			width: 0,
			height: 0,
			tiles: Vec2d::new(Vec::new(), 0),
			position: (0, 0),
			entities: HashSet::default(),
		}
	}
}

/*impl<'a> ProvideReferencesDynamic<'a> for Screen {
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
}*/


