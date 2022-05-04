mod SignalsMod;

pub use SignalsMod::{Signals, Mapping};

struct Vector(f32, f32);

pub struct Player {
	direction: Direction,

	velocity: Vector,
}

impl Player {
    pub fn new() -> Player {
        let (direction, velocity) = (Direction::Down, Vector(0, 0));
        Player {direction, velocity,}
    }
    
    pub fn signal(&mut self, signal: Signals) {
        match signal {
            Signals {up: Some(true), ..} => {
                self.direction = Direction::Up;
                self.velocity.1 = -3;
            },
            Signals {down: Some(true), ..} => {
                self.direction = Direction::Down;
                self.velocity.1 = 3;
            },
            Signals {up: Some(false), ..} => {
                self.velocity.1 = 0;
            },
            Signals {down: Some(false), ..} => {
                self.velocity.1 = 0;
            },
            _ => (),
        }
        match signal {
            Signals {left: Some(true), ..} => {
                self.direction = Direction::Left;
                self.velocity.0 = -3;
            },
            Signals {right: Some(true), ..} => {
                self.direction = Direction::Right;
                self.velocity.0 = 3;
            },
            Signals {left: Some(false), ..} => {
                self.velocity.0 = 0;
            },
            Signals {right: Some(false), ..} => {
                self.velocity.0 = 0;
            },
            _ => (),
        }
    }
}

enum Direction {
	Up,
	Down,
	Left,
	Right,
}

