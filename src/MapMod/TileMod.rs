//use BinaryFileIO::BinaryDataContainer::SelfContained;
//use BinaryFileIO::BFStream::Extend;

use serde::{Serialize, Deserialize};

use sdl2::rect::Rect;

use crate::{ID, Vector};

use super::Map;

pub const OOB: Tile = Tile::OOB();

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tile (u16, CollisionType);

pub struct TileBuilder {
	id: u16,
    collisionType: usize,
    pos: (u16, u16),
	mapId: Option<usize>,
	location: Option<(u16, u16)>,
	locationEnd: Option<(u16, u16)>,
	complete: Option<Tile>,
	entity: Option<ID>,
}

pub enum TileBuilderSignals {
	GetUserUsize(&'static str),
	GetCoordinate(&'static str),
	Complete(Tile, (u16, u16)),
	GetEntity(&'static str),
	InvalidId,
}

impl Tile {
	pub fn new(id: u16, collision: CollisionType) -> Tile {
	    Tile(id, collision)
    }
    pub fn preview(id: u16) -> Tile {
        Tile(id, CollisionType::None)
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
    pub fn gate() -> Tile {
        Tile::new(2, CollisionType::Block)
    }
	pub fn abyss() -> Tile {
		Tile::new(18, CollisionType::Abyss)
	}
}

impl Default for Tile {
	fn default() -> Tile {
		Tile(0, CollisionType::None)
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
	SpawnGate((u16, u16, u16, u16)),
    Hit(i32), //Hurt the player
	Burn, //Burn the player
	ClearTiles((u16, u16, u16, u16)),
	SwitchToggleGate((u16, u16, u16, u16)),
	SwitchTriggerGen(ID),
	Key,
	KeyBlock,
	Abyss,
	SnakeKill,
	SwitchImmune,
	Health,
	TriggerGen(ID),
	SwitchToggleGateAbyss((u16, u16, u16, u16)),
	CannonSword,
	Win,
	OOB, //Represent tiles with oob coordinates
}

pub const COLLISION_NAMES: &'static [&'static str] = &[
    "None",
    "Block",
    "Transition",
    "Hit",
    "Burn",
    "ClearTiles",
    "SpawnGate",
    "SwitchToggleGate",
	"SwitchTriggerGen",
	"Key",
	"KeyBlock",
	"Abyss",
	"SnakeKill",
	"SwitchImmune",
	"Health",
	"TriggerGen",
	"SwitchToggleGateAbyss",
	"CannonSword",
	"Win",
    "OOB",
];

pub const MAX_COLLISION_IDX: usize = COLLISION_NAMES.len() - 2;

impl TileBuilder {
	pub fn new(id: u16, collisionType: usize, pos: (u16, u16)) -> TileBuilder {
		TileBuilder {
			id,
            collisionType,
            pos,
			mapId: None,
			location: None,
			locationEnd: None,
			complete: None,
			entity: None,
		}
	}
	pub fn fromTile(tile: &Tile, pos: (u16, u16)) -> TileBuilder {
		TileBuilder {
			id: 0,
            collisionType: 0,
			pos,
			mapId: None,
			location: None,
			locationEnd: None,
			complete: Some(tile.clone()),
			entity: None,
		}

	}
	pub fn build(&self) -> TileBuilderSignals {
		if let Some(ref tile) = self.complete {
			return TileBuilderSignals::Complete(tile.clone(), self.pos);
		}
        match self.collisionType {
            0 => TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::None), self.pos),
            1 => TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::Block), self.pos),
            2 => {
				if let Some(id) = self.mapId {
					TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::Transition(id)), self.pos)
				}
				else {
					TileBuilderSignals::GetUserUsize("Enter the map id to transition to: ")
				}
			},
            3 => TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::Hit(0)), self.pos),
            4 => TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::Burn), self.pos),

            5 => self.createLocationTile((
                "Click where to begin clearing",
                "Click where to stop clearing",
            ), |(x, y), (xx, yy)| CollisionType::ClearTiles((x, y, xx, yy))),
            6 => self.createLocationTile((
                "Click where the gate begins",
                "Click where the gate ends",
            ), |(x, y), (xx, yy)| CollisionType::SpawnGate((x, y, xx, yy))),
            7 => self.createLocationTile((
                "Click where the gate begins",
                "Click where the gate ends",
            ), |(x, y), (xx, yy)| CollisionType::SwitchToggleGate((x, y, xx, yy))),
			8 => {
				if let Some(entity) = self.entity {
					TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::SwitchTriggerGen(entity)), self.pos)
				}
				else {
					TileBuilderSignals::GetEntity("Pick generator")
				}
			},
			9 => {TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::Key), self.pos)},
			10 => {TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::KeyBlock), self.pos)},
			11 => {TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::Abyss), self.pos)},
			12 => {TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::SnakeKill), self.pos)},
			13 => {TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::SwitchImmune), self.pos)},
			14 => {TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::Health), self.pos)},
			15 => {
				if let Some(entity) = self.entity {
					TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::TriggerGen(entity)), self.pos)
				}
				else {
					TileBuilderSignals::GetEntity("Pick generator")
				}
			},
			16 => self.createLocationTile((
				"Click where the gate begins",
				"Click where the gate ends",
			), |(x, y), (xx, yy)| CollisionType::SwitchToggleGateAbyss((x, y, xx, yy))),
			17 => {TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::CannonSword), self.pos)},
			18 => {TileBuilderSignals::Complete(Tile::new(self.id, CollisionType::Win), self.pos)},
            _ => TileBuilderSignals::InvalidId,
        }
	}
    fn createLocationTile<F: FnOnce((u16, u16), (u16, u16)) -> CollisionType>(&self, msg: (&'static str, &'static str), f: F) -> TileBuilderSignals {
        if let (Some(location), Some(locationEnd)) = (self.location, self.locationEnd) {
            TileBuilderSignals::Complete(Tile::new(self.id, f(location, locationEnd)), self.pos)
        }
        else if let Some(_) = self.location {
            TileBuilderSignals::GetCoordinate(msg.1)
        }
        else {
            TileBuilderSignals::GetCoordinate(msg.0)
        }
    }
	pub fn addUsize(&mut self, num: usize) {
		match self.collisionType {
			2 => self.mapId = Some(num),
			_ => (),
		}
	}
	pub fn addLocation(&mut self, location: (u16, u16)) {
		match self.collisionType {
			5..=7 | 16 if None == self.location => self.location = Some(location),
			5..=7 | 16 => self.locationEnd = Some(location),
			_ => (),
		};
	}
	pub fn addGenerator(&mut self, entity: ID) {
		self.entity = Some(entity);
	}
/*	pub fn cloneTile(&self, tile: &Tile) -> Tile {
		match tile.0 {
			_ => tile.clone()
		}
	}*/
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

pub fn spawnTiles(tile: Tile, location: (u16, u16), locationEnd: (u16, u16), map: &mut Map) {
    for x in location.0..=locationEnd.0 {
        for y in location.1..=locationEnd.1 {
            map.changeTile((x, y), tile.clone());
       }
    }
}

pub const MAX_TILE_IDX: u16 = 23;

