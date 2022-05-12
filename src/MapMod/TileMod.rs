use BinaryFileIO::BinaryDataContainer::SelfContained;
use BinaryFileIO::BFStream::Extend;

#[derive(Clone)]
pub struct Tile (u16, CollisionType);

impl Tile {
	pub fn new(id: u16, object: usize) -> Result<Tile, &'static str> {
		match id {
			0 => Ok(Tile(0, CollisionType::None)),
			1 => Ok(Tile(1, CollisionType::Block)),
			2 => Ok(Tile(2, CollisionType::Block)),
			_ => Err("Recieved invalid tile id"),
		}
	}
	pub fn getId(&self) -> u16 {
		self.0
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
#[derive(Clone)]
pub enum CollisionType {
	None, //Do nothing
	Block, //Block the player
	Switch(usize), //Collision type for switches
	Hit, //Hurt the player
	Burn, //Burn the player
}

pub const MAX_TILE_IDX: u16 = 2;

unsafe impl SelfContained for Tile {}

impl Extend for Tile {}

