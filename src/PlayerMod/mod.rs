extern crate sdl2;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use std::io;

mod SignalsMod;

pub use SignalsMod::{SignalsBuilder, Signals, Mapping};

use crate::SpriteLoader::{Animations, Sprites};
use crate::{Direction, Map, CollisionType, Vector, GameContext};
use crate::Entities::Traits::{Collision, EntityTraitsWrappable, Entity};
use crate::Entities::{BoxCode, RefCode, RefCodeMut};
use crate::EventProcessor::{CollisionMsg, Envelope, PO};
use crate::MapMod::{blockCollide, sharpBlockCollide};

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

const SWORD_FRAMES: &'static[&'static str] = &[
	"Resources/Images/Sword__half.png",
];

const SWORD_DOWN: (i32, i32, u32, u32) = (10, 43, 30, 30);
const SWORD_RIGHT: (i32, i32, u32, u32) = (30, 5, 30, 30);
const SWORD_LEFT: (i32, i32, u32, u32) = (-10, 5, 30, 30);
const SWORD_UP: (i32, i32, u32, u32) = (0, -10, 50, 50);

const SWORD_DOWN_COLLISION: (i32, i32, u32, u32) = (23, 43, 4, 16);
const SWORD_RIGHT_COLLISION: (i32, i32, u32, u32) = (43, 5, 4, 16);
const SWORD_LEFT_COLLISION: (i32, i32, u32, u32) = (3, 5, 4, 16);
const SWORD_UP_COLLISION: (i32, i32, u32, u32) = (27, -10, 6, 27);

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
	attackTimer: u32,
	attacking: bool,
	sword: Sprites<'a>,
}

pub struct PlayerData {
	//transition: Option<Rect>,
	nextPos: Vector,
}

impl PlayerData {
	fn doCollision(&mut self, player: &Player, map: &Map) {
		let mut tmp = player.hitbox;
		tmp.reposition(self.nextPos + Vector(2f32, 2f32));
		let mut iter = map.calculateCollisionBounds(tmp);

		while let Some((location, tile)) = map.collide(&mut iter) {
			match tile.getCollisionType() {
				CollisionType::Block => {
					self.nextPos += blockCollide(location, tmp, map);
					tmp.reposition(self.nextPos);
				},
     /*           CollisionType::SharpBlock => {
					self.nextPos += sharpBlockCollide(location, tmp);
                },*/
				_ => (),
			}
		}
	}
}

impl<'a> Player<'a> {
    pub fn new(creator: &'a TextureCreator<WindowContext>, positionX: f32, positionY: f32) -> io::Result<BoxCode<'a>> {
        let (direction, velocity, position, timer, idle, attackTimer, attacking) = (
            Direction::Down, 
            Vector(0f32, 0f32), 
            Vector(positionX, positionY),
			0u32,
			true,
			0u32,
			false,
        );
		let animations = Animations::new("Resources/Images/Ninja.anim", NAMES, creator)?;
		let sword = Sprites::new(creator, SWORD_FRAMES)?;
		let renderPosition = Rect::new(positionX.round() as i32, positionY.round() as i32, 50, 50);
		let hitbox = Rect::new(positionX.round() as i32 + 2, positionY as i32 + 2, 46, 46);

        Ok(
			BoxCode::Player(
				Box::new(
					Entity::new(
						Player {animations, direction, velocity, position, timer, idle, hitbox, renderPosition, attackTimer, sword, attacking},
						PlayerData {
							nextPos: position,
						},
					)
				)
			)
		)
    }

	fn updatePositions(&mut self) {		
		self.renderPosition.reposition(self.position);
		self.hitbox.reposition(self.position + Vector(2f32, 2f32));
	}

	pub fn transition(&mut self, map: &mut Map) {
		if let Some(hitbox) = map.transitionScreen(self.hitbox) {
			let point: (i32, i32) = hitbox.top_left().into();
			self.position = Vector::from(point);
			self.updatePositions();
		}

	}

    pub fn signal(&mut self, signal: Signals) {
		match (signal.attack, self.attacking) {
			(Some(true), false) => {
				self.attackTimer = 21;
				self.attacking = true;
				self.idle = false;
			},
			(Some(false), true) => {
				self.attacking = false;
			},
			(Some(true), true) | (None, _) | (Some(false), false) => {},
		}
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
	pub fn getPosition(&self) -> Vector {
		self.position
	}
}

impl<'a> Collision for Player<'a> {
	fn collide(&mut self, msg: Envelope<CollisionMsg>) {}
}

impl<'a> EntityTraitsWrappable<'a> for Player<'a> {
	type Data = PlayerData;
	fn mapCodeMut(code: RefCodeMut<'a>) -> Option<&'a mut Self> {
		if let RefCodeMut::Player(p) = code {Some(p as &mut Self)}
		else {None}
	}
	fn mapCode(code: RefCode<'a>) -> Option<&'a Self> {
		if let RefCode::Player(p) = code {Some(p as &Self)}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, ctx: &GameContext) {
		//data.transition = ctx.getMap().transitionScreen(self.hitbox);
		data.nextPos = if self.idle {
			self.position + self.velocity
		} else {self.position};
		data.doCollision(self, ctx.getMap());
	}
	fn update(&mut self, data: &Self::Data, _po: &PO) {
		self.position = data.nextPos;
		self.updatePositions();


		if self.idle{match self.direction {
			Direction::Up => {self.animations.changeAnimation(ANIMATION_IDX::UpFloat as usize);},
			Direction::Down => {self.animations.changeAnimation(ANIMATION_IDX::DownFloat as usize);},
			Direction::Left => {self.animations.changeAnimation(ANIMATION_IDX::LeftFloat as usize);},
			Direction::Right => {self.animations.changeAnimation(ANIMATION_IDX::RightFloat as usize);},
		}}

		self.timer += 1;
		if self.timer > 20 {
			self.timer = 0;
			self.animations.update();
		}
		if self.attackTimer > 0 {
			self.attackTimer -= 1;
		}	
		if self.attackTimer > 0 || self.attacking {
			match self.direction {
				Direction::Up => {self.animations.changeAnimation(ANIMATION_IDX::UpAttack as usize);},
				Direction::Down => {self.animations.changeAnimation(ANIMATION_IDX::DownAttack as usize);},
				Direction::Left => {self.animations.changeAnimation(ANIMATION_IDX::LeftAttack as usize);},
				Direction::Right => {self.animations.changeAnimation(ANIMATION_IDX::RightAttack as usize);},
			};
			// Add attack code here
		}
		self.idle = !self.attacking && self.attackTimer == 0;
	}
	fn needsExecution(&self) -> bool {true}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		if self.attacking || self.attackTimer > 0 {match self.direction {
			Direction::Up => {
				self.sword.getSprite(0).draw(canvas, Rect::new (
					SWORD_UP.0 + self.renderPosition.x(),
					SWORD_UP.1 + self.renderPosition.y(),
					SWORD_UP.2,
					SWORD_UP.3
				), false, false);
				self.animations.drawNextFrame(canvas, self.renderPosition);
			},
			Direction::Down => {
				self.animations.drawNextFrame(canvas, self.renderPosition);
				self.sword.getSprite(0).draw(canvas, Rect::new(
					SWORD_DOWN.0 + self.renderPosition.x(),
					SWORD_DOWN.1 + self.renderPosition.y(),
					SWORD_DOWN.2,
					SWORD_DOWN.3
				), false, true);
			},
			Direction::Left => {
				self.sword.getSprite(0).draw(canvas, Rect::new (
					SWORD_LEFT.0 + self.renderPosition.x(),
					SWORD_LEFT.1 + self.renderPosition.y(),
					SWORD_LEFT.2,
					SWORD_LEFT.3
				), false, false);
				self.animations.drawNextFrame(canvas, self.renderPosition);
			},
			Direction::Right => {
				self.sword.getSprite(0).draw(canvas, Rect::new (
					SWORD_RIGHT.0 + self.renderPosition.x(),
					SWORD_RIGHT.1 + self.renderPosition.y(),
					SWORD_RIGHT.2,
					SWORD_RIGHT.3
				), false, false);
				self.animations.drawNextFrame(canvas, self.renderPosition);
					
			}
		}}
		else {
			self.animations.drawNextFrame(canvas, self.renderPosition);
		}
	}

}

