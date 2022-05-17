extern crate sdl2;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::io;

mod SignalsMod;

pub use SignalsMod::{SignalsBuilder, Signals, Mapping};

use crate::SpriteLoader::Animations;
use crate::{Direction, Map, CollisionType};

const NAMES: &'static[&'static str] = &[
	"Ninja float",
	"Ninja right float",
	"Ninja left float",
	"Ninja up float",
	"Ninja attack",
	"Ninja right attack",
	"Ninja left attack",
	"Ninja up attack",
];

#[derive(Copy, Clone)]
struct Vector(f32, f32);

impl Add for Vector {
    type Output = Vector;
    fn add(self, other: Self) -> Self::Output {
        Vector(self.0 + other.0, self.1 + other.1)
    }
}

impl AddAssign for Vector {
	fn add_assign(&mut self, other: Self) {
		*self = *self + other;
	}
}

impl Sub for Vector {
	type Output = Vector;
	fn sub(self, other: Self) -> Self::Output {	
        Vector(self.0 - other.0, self.1 - other.1)
	}
}

impl SubAssign for Vector {
	fn sub_assign(&mut self, other: Self) {
		*self = *self - other;
	}
}

impl From<Vector> for (i32, i32) {
	fn from(input: Vector) -> (i32, i32) {
		(input.0.round() as i32, input.1.round() as i32)
	}
}

pub struct Player<'a> {
	animations: Animations<'a>,
	direction: Direction,
	timer: u32,

	velocity: Vector,
    position: Vector,
	hitbox: Rect,
	renderPosition: Rect,
}

impl<'a> Player<'a> {
    pub fn new(creator: &'a TextureCreator<WindowContext>, positionX: f32, positionY: f32) -> io::Result<Player<'a>> {
        let (direction, velocity, position, timer) = (
            Direction::Down, 
            Vector(0f32, 0f32), 
            Vector(positionX, positionY),
			0u32,
        );
		let animations = Animations::new("Resources/Images/Ninja.anim", NAMES, creator)?;
		let renderPosition = Rect::new(positionX.round() as i32, positionY.round() as i32, 50, 50);
		let hitbox = Rect::new(positionX.round() as i32 + 2, positionY as i32 + 2, 46, 46);

        Ok(Player {animations: animations, direction, velocity, position, timer, hitbox, renderPosition,})
    }

	pub fn draw(&self, canvas: &mut Canvas<Window>) {
		self.animations.drawNextFrame(canvas, self.renderPosition);
	}

	fn updatePositions(&mut self) {		
		self.renderPosition.reposition(self.position);
		self.hitbox.reposition(self.position + Vector(2f32, 2f32));
	}
	fn doCollision(&mut self, map: &mut Map) {
		let mut iter = Map::calculateCollisionBounds(self.hitbox);

		let mut blocked = false;

		while let Some(tile) = map.collide(&mut iter) {
			match tile.getCollisionType() {
				CollisionType::Block if !blocked => {
					self.position -= self.velocity;	
					self.updatePositions();
					blocked = true;
				}
				_ => (),
			}
		}
	}

    pub fn update(&mut self, map: &mut Map) {
		let tmp = self.velocity;
        self.velocity.1 = 0f32;
		self.position += self.velocity;
		self.updatePositions();
		self.doCollision(map);

		self.velocity = tmp;
		self.velocity.0 = 0f32;
		self.position += self.velocity;
		self.updatePositions();
		self.doCollision(map);

		self.velocity = tmp;


		self.timer += 1;
		if self.timer > 20 {
			self.timer = 0;
			self.animations.update();
		}
    }
    
    pub fn signal(&mut self, signal: Signals) {
        match signal {
            Signals {up: Some(true), ..} => {
                self.direction = Direction::Up;
                self.velocity.1 = -3f32;
            },
            Signals {down: Some(true), ..} => {
                self.direction = Direction::Down;
                self.velocity.1 = 3f32;
            },
            Signals {up: Some(false), ..} => {
                self.velocity.1 = 0f32;
            },
            Signals {down: Some(false), ..} => {
                self.velocity.1 = 0f32;
            },
            _ => (),
        }
        match signal {
            Signals {left: Some(true), ..} => {
                self.direction = Direction::Left;
                self.velocity.0 = -3f32;
            },
            Signals {right: Some(true), ..} => {
                self.direction = Direction::Right;
                self.velocity.0 = 3f32;
            },
            Signals {left: Some(false), ..} => {
                self.velocity.0 = 0f32;
            },
            Signals {right: Some(false), ..} => {
                self.velocity.0 = 0f32;
            },
            _ => (),
        }
    }
}

