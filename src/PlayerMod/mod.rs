extern crate sdl2;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use std::io;

mod SignalsMod;

pub use SignalsMod::{SignalsBuilder, Signals, Mapping};

use crate::SpriteLoader::Animations;
use crate::{Direction, Map, CollisionType, Vector, GameContext};
use crate::Entities::Traits::{Collision, EntityTraitsWrappable};
use crate::Entities::RefCode;
use crate::EventProcessor::{CollisionMsg, Envelope};

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

enum ANIMATION_IDX {
	DownFloat = 0,
	RightFloat,
	LeftFloat,
	UpFloat,
	DownAttack,
	RightAttack,
	LeftAttack,
	UpAttack,
}

pub struct Player<'a> {
	animations: Animations<'a>,
	direction: Direction,
	timer: u32,
	idle: bool,
	velocity: Vector,
    position: Vector,
	hitbox: Rect,
	renderPosition: Rect,
}

pub struct PlayerData {

}

impl<'a> Player<'a> {
    pub fn new(creator: &'a TextureCreator<WindowContext>, positionX: f32, positionY: f32) -> io::Result<Player<'a>> {
        let (direction, velocity, position, timer, idle) = (
            Direction::Down, 
            Vector(0f32, 0f32), 
            Vector(positionX, positionY),
			0u32,
			true,
        );
		let animations = Animations::new("Resources/Images/Ninja.anim", NAMES, creator)?;
		let renderPosition = Rect::new(positionX.round() as i32, positionY.round() as i32, 50, 50);
		let hitbox = Rect::new(positionX.round() as i32 + 2, positionY as i32 + 2, 46, 46);

        Ok(Player {animations, direction, velocity, position, timer, idle, hitbox, renderPosition,})
    }

	pub fn draw(&self, canvas: &mut Canvas<Window>) {
		self.animations.drawNextFrame(canvas, self.renderPosition);
	}

	fn updatePositions(&mut self) {		
		self.renderPosition.reposition(self.position);
		self.hitbox.reposition(self.position + Vector(2f32, 2f32));
	}
	fn doCollision(&mut self, map: &mut Map) {
		let mut iter = map.calculateCollisionBounds(self.hitbox);

		while let Some((location, tile)) = map.collide(&mut iter) {
			match tile.getCollisionType() {
				CollisionType::Block => {
					let ejectionDirection = Vector::fromPoints((location.0 as f32 * 50f32, location.1 as f32 * 50f32), self.position);
					let (mut x, mut y) = (0f32, 0f32);
					if ejectionDirection.0.abs() > ejectionDirection.1.abs() {
						x = (50f32 - ejectionDirection.0.abs()) * ejectionDirection.0.signum();
					}
					if ejectionDirection.0.abs() < ejectionDirection.1.abs() {
						y = (50f32 - ejectionDirection.1.abs()) * ejectionDirection.1.signum();
					}
					let ejectionVector = Vector(x, y);
					self.position += ejectionVector;	
					self.updatePositions();
				},
                CollisionType::SharpBlock => {
					let ejectionDirection = Vector::fromPoints((location.0 as f32 * 50f32, location.1 as f32 * 50f32), self.position);
					let (mut x, mut y) = (0f32, 0f32);
					if ejectionDirection.0.abs() >= ejectionDirection.1.abs() {
						x = (50f32 - ejectionDirection.0.abs()) * ejectionDirection.0.signum();
					}
					if ejectionDirection.0.abs() <= ejectionDirection.1.abs() {
						y = (50f32 - ejectionDirection.1.abs()) * ejectionDirection.1.signum();
					}
					let ejectionVector = Vector(x, y);
					self.position += ejectionVector;	
					self.updatePositions();
                },
				_ => (),
			}
		}
	}

    pub fn update(&mut self, map: &mut Map) {
		self.position += self.velocity;
		self.updatePositions();

		if let Some(hitbox) = map.transitionScreen(self.hitbox) {
			let point: (i32, i32) = hitbox.top_left().into();
			self.position = Vector::from(point);
			self.updatePositions();
		}

		if self.idle{match self.direction {
			Direction::Up => {self.animations.changeAnimation(ANIMATION_IDX::UpFloat as usize);},
			Direction::Down => {self.animations.changeAnimation(ANIMATION_IDX::DownFloat as usize);},
			Direction::Left => {self.animations.changeAnimation(ANIMATION_IDX::LeftFloat as usize);},
			Direction::Right => {self.animations.changeAnimation(ANIMATION_IDX::RightFloat as usize);},
		}}

		self.doCollision(map);

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

impl<'a> Collision for Player<'a> {
	fn collide(&mut self, msg: Envelope<CollisionMsg>) {}
}

impl<'a> EntityTraitsWrappable<'a> for Player<'a> {
	type Data = PlayerData;
	fn mapCode(code: RefCode<'a>) -> Option<&'a mut Self> {
		if let RefCode::Player(p) = code {Some(p as &mut Self)}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, ctx: &GameContext) {}
	fn update(&mut self, data: &Self::Data) {}
}

