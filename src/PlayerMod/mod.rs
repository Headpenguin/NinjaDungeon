mod SignalsMod;

use std::ops::Add;

pub use SignalsMod::{Signals, Mapping};

struct Vector(f32, f32);

impl Add for Vector {
    type Rhs = Vector;
    fn add(&self, other: Rhs) -> Self {
        Vector(self.0 + other.0, self.1 + other.1)
    }
}

pub struct Player {
	direction: Direction,

	velocity: Vector,
    position: Vector,
}

impl Player {
    pub fn new(positionX: f32, positionY: f32) -> Player {
        let (direction, velocity, position) = (
            Direction::Down, 
            Vector(0, 0), 
            Vector(positionX, positionY)
        );

        Player {direction, velocity, position,}
    }

    pub fn update(&mut self) {
        self.position += self.velocity;
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

