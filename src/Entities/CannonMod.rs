use serde::{Serialize, Deserialize};

use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

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

const CANNONBALL: &'static [&'static str] = &["Resources/Images/CannonBall.png"];

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
struct CannonBall {
	pos: Vector,
	hitbox: Rect,
	velocity: Vector,
	renderPosition: Rect,
	die: bool,
	timer: u16,
}

impl CannonBall {
	fn new(pos: Vector, velocity: Vector) -> CannonBall {
		CannonBall {
			pos,
			hitbox: Rect::new(pos.0 as i32, pos.1 as i32, 15, 15),
			velocity,
			renderPosition: Rect::new(pos.0 as i32, pos.1 as i32, 15, 15),
			die: false,
			timer: 600,
		}
	}
	fn update(&mut self) {
		self.pos += self.velocity * 2f32;
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
}

#[derive(Debug, Default)]
pub struct CannonData {
	spawnBall: Option<Vector>,
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
}

impl<'a> Collision for Cannon<'a> {
	fn collide(&mut self, msg: Envelope<CollisionMsg>, po: &PO) {
		let i = msg.getReciever().getSubID();
		if i != 0 {
			if let CollisionMsg::Damage(_) = msg.getMsg() {
				self.cannonsBalls[i as usize - 1].as_mut().unwrap().die = true;
			}
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
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key {
		data.spawnBall = None;
		let playerPos = po.getCtx().getHolder().getTyped(po.getCtx().getPlayerID()).unwrap().getCenter();
		if Common::checkLineOfSight(self.pos, playerPos - self.pos, po) {
			data.spawnBall = Some(Vector::fromPoints(self.pos, playerPos).normalizeOrZero());
		}
		key
	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
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
	}
	fn needsExecution(&self) -> bool {true}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		self.animations.drawNextFrame(canvas, self.renderPosition);
		for ball in self.cannonsBalls.iter().filter_map(|e| e.as_ref()) {
			self.cannonballSprites.getSprite(0).draw(canvas, ball.renderPosition, false, false);
		}
	}
}

