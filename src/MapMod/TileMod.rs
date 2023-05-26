//use BinaryFileIO::BinaryDataContainer::SelfContained;
//use BinaryFileIO::BFStream::Extend;

use serde::{Serialize, Deserialize};

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
			4 => Ok(Tile(1, CollisionType::SharpBlock)),
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
    SharpBlock, //Block the player and push diagonally at corners
	Transition(usize),
	Switch(usize), //Collision type for switches
	Hit, //Hurt the player
	Burn, //Burn the player
}

pub const MAX_TILE_IDX: u16 = 4;

//unsafe impl SelfContained for Tile {}

//impl Extend for Tile {}

