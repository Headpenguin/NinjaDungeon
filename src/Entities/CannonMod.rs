use serde::{Serialize, Deserialize};

use sdl2::rect::{Rect, Point};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use rand::prelude::*;

use std::io;

use super::Traits::{Collision, EntityTraitsWrappable, Entity, Counter, RegisterID, IDRegistration};
use super::{BoxCode, RefCode, RefCodeMut, TypedID};
use super::Common::{DeathCounter, self};
use crate::SpriteLoader::Animations;
use crate::SpriteLoader::Sprites;
use crate::{Vector, ID, Direction};
use crate::EventProcessor::{CollisionMsg, Envelope, PO, Key};
use crate::CollisionType;
use crate::MapMod;

const NAMES: &'static [&'static str] = &[
	"CannonWalkDown",
	"CannonWalkLeft",
	"CannonWalkRight",
	"CannonWalkUp",
];

pub const CANNONBALL: &'static [&'static str] = &["Resources/Images/CannonBall.png"];

enum ANIMATIONS_IDX {
	WalkDown = 0,
	WalkLeft,
	WalkRight,
	WalkUp,
}

#[derive(Serialize, Deserialize)]
pub struct InnerCannon {
	id: ID,
	pos: Vector,
	variant: u8,
	deathEvent: Option<DeathCounter>,
}

impl InnerCannon {
	pub fn fromCannon(&Cannon { id, pos, variant, deathEvent, .. }: &Cannon) -> Self {
		InnerCannon {id: id.getID(), pos, variant, deathEvent}
	}
}

#[derive(Debug)]
pub struct CannonBall {
	pos: Vector,
	pub hitbox: Rect,
	velocity: Vector,
	pub renderPosition: Rect,
	pub die: bool,
	timer: u16,
}

impl CannonBall {
	pub fn new(pos: Vector, velocity: Vector) -> CannonBall {
		CannonBall {
			pos,
			hitbox: Rect::new(pos.0 as i32, pos.1 as i32, 15, 15),
			velocity,
			renderPosition: Rect::new(pos.0 as i32, pos.1 as i32, 15, 15),
			die: false,
			timer: 600,
		}
	}
	pub fn update(&mut self) {
		self.pos += self.velocity * 4f32;
		self.hitbox = Rect::new(self.pos.0 as i32, self.pos.1 as i32, 15, 15);
		self.renderPosition = self.hitbox;
		self.timer -= 1;
		if self.timer == 0 {self.die = true;}
	}
}

#[derive(Debug)]
pub struct Cannon<'a> {
	id: TypedID<'a, Self>,
	animations: Animations<'a>,
	cannonballSprites: Sprites<'a>,
	cannonsBalls: [Option<CannonBall>; 3],
	pos: Vector,
	hitbox: Rect,
	renderPosition: Rect,
	timer: u16,
	variant: u8,
	deathEvent: Option<DeathCounter>,
	idle: bool,
	stateTimer: u16,
	dir: Direction,
	groundVelocity: Vector,
	elevated: u8,
	health: i32,
}

#[derive(Debug, Default)]
pub struct CannonData {
	spawnBall: Option<Vector>,
	pos: Vector,
}

impl<'a> Cannon<'a> {
	fn newInt(creator: &'a TextureCreator<WindowContext>, pos: Vector, variant: u8, deathEvent: Option<DeathCounter>) -> io::Result<Self> {
		Ok(Cannon {
			id: TypedID::new(ID::empty()),
			animations: Animations::new("Resources/Images/Cannon.anim", NAMES, creator)?,
			cannonballSprites: Sprites::new(creator, CANNONBALL)?,
			cannonsBalls: [None, None, None],
			pos,
			hitbox: Rect::new(pos.0 as i32, pos.1 as i32, 50, 50),
			renderPosition: Rect::new(pos.0 as i32, pos.1 as i32, 50, 50),
			variant,
			deathEvent,
			timer: 0,
			idle: false,
			dir: Direction::Down,
			stateTimer: 10,
			groundVelocity: Vector(0f32, 0f32),
			elevated: 0,
			health: if variant == 1 {50} else {20},
		})
	}
	pub fn new(creator: &'a TextureCreator<WindowContext>, pos: Vector) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Cannon(
			Entity::new(
				Self::newInt(creator, pos, 0, None)?,
				CannonData::default()
			)
		))
	}
	pub fn fromInner(InnerCannon { id, pos, variant, deathEvent }: InnerCannon, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		let mut tmp = Self::newInt(creator, pos, variant, deathEvent)?;
		tmp.setID(TypedID::new(id));
		Ok(BoxCode::Cannon(
			Entity::new(
				tmp,
				CannonData::default()
			)
		))
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.hitbox.has_intersection(hitbox)
	}
	fn playerIsInDirection(&self, playerPos: Vector) -> bool {
		let info = (playerPos.0 > self.pos.0, playerPos.1 > self.pos.1);
		match self.dir {
			Direction::Up => !info.1,
			Direction::Down => info.1,
			Direction::Left => !info.0,
			Direction::Right => info.0,
		}
	}
	fn updatePositions(&mut self, po: &mut PO) {
		let oldPos = self.hitbox;
		self.hitbox.reposition(self.pos);
		self.renderPosition.reposition(self.pos);
		po.updatePosition(self.id.getID(), self.hitbox, oldPos);
	}
	fn calcTrajectory(&self, playerPos: Vector, playerVelocity: Vector) -> Vector {
		/*println!("{:?}", playerVelocity);
		let playerPos = playerPos - self.pos;
		let center = playerVelocity.0 * playerPos.0 + playerPos.1 * playerVelocity.1;
		let coefficient = 4f32 - playerVelocity.mag().powi(2);
		if coefficient == 0f32 {return Vector(0f32, 0f32);}
		let vertex = playerPos.mag().powi(2) / coefficient + (center / coefficient).powi(2);
		if vertex < 0f32 {return Vector(0f32, 0f32);}
		let t = vertex.sqrt() + center / coefficient;
		let playerPos = playerPos + playerVelocity * t;
		
		if playerPos.0 == 0f32 {
			Vector(0f32, 1f32)
		}
		else {
			let m = playerPos.1 / playerPos.0;
			Vector(1f32, m)
		}*/
		let playerPos = playerPos - self.pos;
		let a = playerVelocity.dot(&playerVelocity) - 16f32;
		let b = playerVelocity.dot(&playerPos);
		let c = playerPos.dot(&playerPos);
		let d = b*b - 16f32*a*c;
		if d < 0f32 || a == 0f32 {
			playerPos
		}
		else {
			let t = (-b - d.sqrt())/(2f32 * a);
			let t = if t < 0f32 {(-b + d.sqrt())/(2f32 * a)} else {t};
			playerPos + playerVelocity * t
		}
	}
}

impl<'a> Collision for Cannon<'a> {
	fn collide(&mut self, msg: Envelope<CollisionMsg>, po: &PO) {
		let i = msg.getReciever().getSubID();
		if i != 0 {
			if let CollisionMsg::Damage(_) = msg.getMsg() {
				self.cannonsBalls[i as usize - 1].as_mut().unwrap().die = true;
			}
		}
		else {
			match msg.getMsg() {
				CollisionMsg::Ground(hitbox, dp) => {
					let displacement = hitbox.center() - self.hitbox.center();
					if displacement.x.abs() <= 15 && displacement.y.abs() <= 15 {
						if self.elevated == 0 {
							self.pos = Vector::from(<Point as Into<(i32, i32)>>::into(hitbox.top_left()));
						}
						else {
							self.groundVelocity = *dp;
						}
						self.elevated = 2;
					}
				}
				CollisionMsg::Damage(dmg) => {
					let sender = msg.getSender();
					if sender.mask() == po.getCtx().getPlayerID().getID().mask() && sender.getSubID() <= 4 && sender.getSubID() >= 2 {
						self.health -= dmg;
						po.sendCollisionMsg(Envelope::new(CollisionMsg::Damage(1), sender, self.id.getID()));
					}
				},
			};
		}
	}
	fn collideWith(&self, id: ID, other: ID, po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {
		let i = id.getSubID();
		if i == 0 {(None, key)}
		else {
			po.sendCollisionMsg(Envelope::new(CollisionMsg::Damage(-1), id, self.id.getID()));
			(Some(Envelope::new(CollisionMsg::Damage(3), other, id)), key)
		}
	}
}

impl<'a> Counter for Cannon<'a> {}
impl<'a> RegisterID for Cannon<'a> {
	fn register(&mut self, id: IDRegistration) {
		match id {
			IDRegistration::DeathCounter(id) => self.deathEvent = Some(DeathCounter::new(id)),
		};
	}
}

impl<'a> EntityTraitsWrappable<'a> for Cannon<'a> {
	type Data = CannonData;
	fn setID(&mut self, id: TypedID<'a, Self>) {
		self.id = id;
	}
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self> {
		if let RefCodeMut::Cannon(c) = code {Some(c as &mut Self)}
		else {None}
	}
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self> {
		if let RefCode::Cannon(c) = code {Some(c as &Self)}
		else {None}
	}
    fn drawPriority(&self) -> u8 {1}
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key {
		data.spawnBall = None;
		data.pos = Vector(0f32, 0f32);
		let (playerPos, playerVelocity) = {
            let player = po.getCtx().getHolder().getTyped(po.getCtx().getPlayerID()).unwrap();
            (player.getPosition(), player.getVelocity())
        };
		if self.stateTimer == 1 && self.playerIsInDirection(playerPos) && Common::checkLineOfSight(self.pos, playerPos - self.pos, po) {
			data.spawnBall = Some(self.calcTrajectory(playerPos, playerVelocity).normalizeOrZero());
		}
		if !self.idle {
			let mut tmp = self.hitbox;
			data.pos = match self.dir {
				Direction::Up => Vector(0f32, -1.5f32),
				Direction::Down => Vector(0f32, 1.5f32),
				Direction::Left => Vector(-1.5f32, 0f32),
				Direction::Right => Vector(1.5f32, 0f32),
			};
			tmp.reposition(self.pos + data.pos);

			let map = po.getCtx().getMap();
			let mut iter = map.calculateCollisionBounds(tmp);
			
			while let Some((location, tile)) = map.collide(&mut iter) {
				match tile.getCollisionType() {
					CollisionType::KeyBlock | CollisionType::Block | CollisionType::Burn | CollisionType::Abyss | CollisionType::SwitchImmune => {
						data.pos = Vector(0f32, 0f32);
						break;
					}
					_ => (),
				}
			}
		}

		key
	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		if self.health <= 0 {
			if self.variant == 1 {
				po.win();
			}
			po.removeCollision(self.id.getID(), self.hitbox);
			for i in 1..=3 {
				if let Some(ref ball) = self.cannonsBalls[i - 1] {
					po.removeCollision(self.id.getID().sub(i as u8), ball.hitbox);
				}
			}
			po.addToPurgeList(self.id.getID());
		}
		self.pos += data.pos + self.groundVelocity;
		self.updatePositions(po);
		if let Some(velocity) = data.spawnBall {
			if let Some((i, _)) = self.cannonsBalls.iter().enumerate().filter(|b| b.1.is_none()).next() {
				self.cannonsBalls[i] = Some(CannonBall::new(self.pos, velocity));
			}
		}
		for (i, ball) in self.cannonsBalls.iter_mut().enumerate().filter_map(|(i, e)| e.as_mut().map(|e| (i, e))) {
			let oldHitbox = ball.hitbox;
			ball.update();
			po.updatePosition(self.id.getID().sub(i as u8 + 1), ball.hitbox, oldHitbox);
		}
		for (i, ball) in self.cannonsBalls.iter_mut().enumerate() {
			if let Some(ref mut ballInner) = ball {
				if ballInner.die {
					po.removeCollision(self.id.getID().sub(i as u8 + 1), ballInner.hitbox);
					*ball = None;
				}
			}
		}
		self.timer += 1;
		if self.timer > 30 {
			self.timer = 0;
			self.animations.update();
		}
		self.stateTimer -= 1;
		if !self.idle && self.stateTimer == 0 {
			self.idle = true;
			self.stateTimer = (rand::thread_rng().gen::<f32>() * 110f32 + 10f32) as u16;
		}
		if self.idle && self.stateTimer == 0 {
			self.idle = false;
			self.stateTimer = (rand::thread_rng().gen::<f32>() * 60f32 + 60f32) as u16;
			self.dir = match (rand::thread_rng().gen::<f32>() * 4f32) as u8{
				0 => Direction::Up,
				1 => Direction::Down,
				2 => Direction::Left,
				3 => Direction::Right,
				_ => unreachable!(),
			};
			match self.dir {
				Direction::Up => self.animations.changeAnimation(ANIMATIONS_IDX::WalkUp as usize),
				Direction::Down => self.animations.changeAnimation(ANIMATIONS_IDX::WalkDown as usize),
				Direction::Left => self.animations.changeAnimation(ANIMATIONS_IDX::WalkLeft as usize),
				Direction::Right => self.animations.changeAnimation(ANIMATIONS_IDX::WalkRight as usize),
			};
		}
		self.groundVelocity = Vector(0f32, 0f32);
        if self.elevated > 0 {
            self.elevated -= 1;
        }
	}
	fn needsExecution(&self) -> bool {true}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		self.animations.drawNextFrame(canvas, self.renderPosition);
		for ball in self.cannonsBalls.iter().filter_map(|e| e.as_ref()) {
			self.cannonballSprites.getSprite(0).draw(canvas, ball.renderPosition, false, false);
		}
		println!("{:?}", self.renderPosition);
	}
}

