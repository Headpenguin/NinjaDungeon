pub struct Tile (u16, CollisionType);

impl Tile {
	pub fn new(id: u16, object: usize) -> Result<Tile, &'static str> {
		match id {
			0 => Ok(Tile(0, CollisionType::None)),
			_ => Err("Recieved invalid tile id"),
		}
	}
}

/*
 CollisionType can be used for almost anything. Obviously, walls and other damaging tiles,
 but with switches, for example, you merely need to put a box in front of the player for the sword
 and check collision like that. For a sign, check some amount in front of the player (probably
 the same amount as the walking velocity). Also, just add an index to a vector to have additional data.
*/
pub enum CollisionType {
	None, //Do nothing
	Block, //Block the player
	Switch(usize), //Collision type for switches
	Hit, //Hurt the player
	Burn, //Burn the player
}

