//use BinaryFileIO::BinaryDataContainer::SelfContained;
//use BinaryFileIO::BFStream::Extend;

use serde::{Serialize, Deserialize};

use sdl2::rect::{Rect, Point};

use crate::Vector;

use super::Map;

#[derive(Serialize, Deserialize, Clone)]
pub struct Tile (u16, CollisionType);

pub struct TileBuilder {
	id: u16,
	mapId: Option<usize>,
}

pub enum TileBuilderSignals {
	GetUserUsize(&'static str),
	Complete(Tile),
	InvalidId,
}

impl Tile {
	pub fn new(id: u16, object: usize) -> Result<Tile, &'static str> {
		match id {
			0 => Ok(Tile(0, CollisionType::None)),
			1 => Ok(Tile(1, CollisionType::Block)),
			2 => Ok(Tile(2, CollisionType::Block)),
			3 => Ok(Tile(0, CollisionType::Transition(object))),
			_ => Err("Recieved invalid tile id"),
		}
	}
	pub fn getId(&self) -> u16 {
		self.0
	}
	pub fn getCollisionType(&self) -> CollisionType {
		self.1
	}
}

impl Default for Tile {
	fn default() -> Tile {
		Tile(0, CollisionType::None)
	}
}

impl TileBuilder {
	pub fn new(id: u16) -> TileBuilder {
		TileBuilder {
			id,
			mapId: None,
		}
	}
	pub fn build(&self) -> TileBuilderSignals {
		match self.id {
			3 => {
				if let Some(id) = self.mapId {
					TileBuilderSignals::Complete(Tile::new(self.id, id).unwrap())
				}
				else {
					TileBuilderSignals::GetUserUsize("Enter the map id to transition to: ")
				}
			},

			id => if let Ok(tile) = Tile::new(id, 0) {
				TileBuilderSignals::Complete(tile)
			}
			else {
				TileBuilderSignals::InvalidId
			},
		}
	}
	pub fn addUsize(&mut self, num: usize) {
		match self.id {
			3 => self.mapId = Some(num),
			_ => (),
		}
	}
	pub fn cloneTile(&self, tile: &Tile) -> Tile {
		match tile.0 {
			_ => tile.clone()
		}
	}
}

/*
 CollisionType can be used for almost anything. Obviously, walls and other damaging tiles,
 but with switches, for example, you merely need to put a box in front of the player for the sword
 and check collision like that. For a sign, check some amount in front of the player (probably
 the same amount as the walking velocity). Also, just add an index to a vector to have additional data.
*/
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum CollisionType {
	None, //Do nothing
	Block, //Block the player
	Transition(usize),
	Switch(usize), //Collision type for switches
	Hit, //Hurt the player
	Burn, //Burn the player
}

fn determineCollidedSide(sides: (i32, i32, i32, i32)) -> Side {
	//Determine the index of the minimum value
	let arr = [sides.0, sides.1, sides.2, sides.3];
	Side::new(arr.iter().enumerate().min_by(|x, y| x.1.cmp(y.1)).unwrap().0 as u8)
}

enum Side {
	Top,
	Bottom,
	Left,
	Right,
}

impl Side {
	fn new(code: u8) -> Side {
		match code {
			0 => Side::Top,
			1 => Side::Bottom,
			2 => Side::Left,
			3 => Side::Right,
			_ => unreachable!(),
		}
	}
}

pub fn blockCollide(location: (u16, u16), hitbox: Rect, map: &Map) -> Vector {
	let tile = Rect::new(location.0 as i32 * 50, location.1 as i32 * 50, 50, 50);
	let (top, bottom, left, right) = (
		tile.bottom() - hitbox.top(),
		hitbox.bottom() - tile.top(),
		tile.right() - hitbox.left(),
		hitbox.right() - tile.left(),
	);
	if top <= 0 || bottom <= 0 || left <= 0 || right <= 0 {
		return Vector(0f32, 0f32);
	}
	let (location, eject) = match determineCollidedSide((top, bottom, left, right)) {
		Side::Top => ((location.0, location.1 + 1), Vector(0f32, top as f32)),
		Side::Bottom => ((location.0, location.1 - 1), Vector(0f32, -bottom as f32)),
		Side::Left => ((location.0 + 1, location.1), Vector(left as f32, 0f32)),
		Side::Right => ((location.0 - 1, location.1), Vector(-right as f32, 0f32))
	};
	if let CollisionType::Block =
		map.getScreen(map.getActiveScreenId()).unwrap().getTile(location).getCollisionType() {
		Vector(0f32, 0f32)
	}
	else {
		eject
	}
}
/*
pub fn sharpBlockCollide(location: (u16, u16), position: Vector) -> Vector {
	let ejectionDirection = Vector::fromPoints((location.0 as f32 * 50f32, location.1 as f32 * 50f32), position);
	let (mut x, mut y) = (0f32, 0f32);
	if ejectionDirection.0.abs() >= ejectionDirection.1.abs() {
		x = (50f32 - ejectionDirection.0.abs()) * ejectionDirection.0.signum();
	}
	if ejectionDirection.0.abs() <= ejectionDirection.1.abs() {
		y = (50f32 - ejectionDirection.1.abs()) * ejectionDirection.1.signum();
	}
	Vector(x, y)
}*/

pub const MAX_TILE_IDX: u16 = 3;

//unsafe impl SelfContained for Tile {}

//impl Extend for Tile {}

