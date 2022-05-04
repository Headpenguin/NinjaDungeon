mod SignalsMod;

pub use SignalsMod::{Signals, Mapping};

struct Vector(f32, f32);

pub struct Player {
	direction: Direction,

	velocity: Vector,
}

impl Player {

}

enum Direction {
	Up,
	Down,
	Left,
	Right,
}

