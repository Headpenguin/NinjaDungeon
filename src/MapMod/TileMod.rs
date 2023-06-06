//use BinaryFileIO::BinaryDataContainer::SelfContained;
//use BinaryFileIO::BFStream::Extend;

use serde::{Serialize, Deserialize};

use sdl2::rect::Rect;

use crate::Vector;

use super::Map;

pub const OOB: Tile = Tile::OOB();

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tile (u16, CollisionType);

pub struct TileBuilder {
	id: u16,
    pos: (u16, u16),
	mapId: Option<usize>,
	location: Option<(u16, u16)>,
	locationEnd: Option<(u16, u16)>,
}

pub enum TileBuilderSignals {
	GetUserUsize(&'static str),
	GetCoordinate(&'static str),
	Complete(Tile, (u16, u16)),
	InvalidId,
}

impl Tile {
	pub fn new(id: u16, object: usize) -> Result<Tile, &'static str> {
		match id {
			0 => Ok(Tile(0, CollisionType::None)),
			1 => Ok(Tile(1, CollisionType::Block)),
			2 => Ok(Tile(2, CollisionType::Block)),
			3 => Ok(Tile(0, CollisionType::Transition(object))),
			4 => Ok(Tile(0, CollisionType::SpawnGate(((object & 0xffff) as u16, ((object & 0xffff0000) >> 0x10) as u16, ((object & 0xffff00000000) >> 0x20) as u16, ((object & 0xffff000000000000) >> 0x30) as u16)))),
			_ => Err("Recieved invalid tile id"),
		}
	}
	pub const fn OOB() -> Tile {
		Tile(u16::MAX, CollisionType::OOB)
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
	pub fn new(id: u16, pos: (u16, u16)) -> TileBuilder {
		TileBuilder {
			id,
            pos,
			mapId: None,
			location: None,
			locationEnd: None,
		}
	}
	pub fn build(&self) -> TileBuilderSignals {
		match self.id {
			4 => {
				if let (Some(location), Some(locationEnd)) = (self.location, self.locationEnd) {
					TileBuilderSignals::Complete(Tile::new(self.id, location.0 as usize + ((location.1 as usize) << 0x10) + ((locationEnd.0 as usize) << 0x20) + ((locationEnd.1 as usize) << 0x30)).unwrap(), self.pos)
				}
				else if let Some(_) = self.location {
					TileBuilderSignals::GetCoordinate("Click where the gates end")
				}
				else {
					TileBuilderSignals::GetCoordinate("Click where the gate goes")
				}
			}
			3 => {
				if let Some(id) = self.mapId {
					TileBuilderSignals::Complete(Tile::new(self.id, id).unwrap(), self.pos)
				}
				else {
					TileBuilderSignals::GetUserUsize("Enter the map id to transition to: ")
				}
			},

			id => if let Ok(tile) = Tile::new(id, 0) {
				TileBuilderSignals::Complete(tile, self.pos)
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
	pub fn addLocation(&mut self, location: (u16, u16)) {
		match self.id {
			4 if None == self.location => self.location = Some(location),
			4 => self.locationEnd = Some(location),
			_ => (),
		};
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
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum CollisionType {
	None, //Do nothing
	Block, //Block the player
	Transition(usize),
	SpawnGate((u16, u16, u16, u16)), //Collision type for switches
	Hit, //Hurt the player
	Burn, //Burn the player
	
	OOB, //Represent tiles with oob coordinates
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
	
	// Determine distance from each side of the hitbox to the closest side of the tile, with the positive direction being the center of the tile
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
		Side::Bottom => ((location.0, location.1.wrapping_sub(1)), Vector(0f32, -bottom as f32)),
		Side::Left => ((location.0 + 1, location.1), Vector(left as f32, 0f32)),
		Side::Right => ((location.0.wrapping_sub(1), location.1), Vector(-right as f32, 0f32))
	};
	
	if let CollisionType::Block | CollisionType::OOB =
		map.getScreen(map.getActiveScreenId()).unwrap().getTile(location).getCollisionType() {
		Vector(0f32, 0f32)
	}
	
	else {
		eject
	}
}

pub fn placeGate(location: (u16, u16), locationEnd: (u16, u16), map: &mut Map) {
	for x in location.0..=locationEnd.0 {
		for y in location.1..=locationEnd.1 {
			map.changeTile((x, y), Tile::new(2, 0).unwrap());
		}
	}
}

pub const MAX_TILE_IDX: u16 = 4;

