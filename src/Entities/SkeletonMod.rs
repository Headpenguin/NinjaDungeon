use serde::{Serialize, Deserialize};

use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use std::io;

use super::Traits::{Collision, EntityTraitsWrappable, Entity, Counter, RegisterID, IDRegistration};
use super::{BoxCode, RefCode, RefCodeMut, TypedID};
use super::Common::DeathCounter;
use crate::SpriteLoader::Animations;
use crate::{GameContext, Vector, ID};
use crate::EventProcessor::{CollisionMsg, Envelope, PO, Key};
use crate::CollisionType;
use crate::MapMod;


const NAMES_TOP: &'static[&'static str] = &[
	"Skeleton",
];

const NAMES_BOTTOM: &'static[&'static str] = &[
	"Skeleton idle",
	"Skeleton walk 0",
	"Skeleton walk 1",
];

enum ANIMATION_IDX_TOP {
	Skeleton = 0,
}

enum ANIMATION_IDX_BOTTOM {
	Idle = 0,
	Walk0,
	Walk1,
}

#[derive(Serialize, Deserialize)]
pub struct InnerSkeleton {
	id: ID,
	timer: u32,
	renderPositionTop: (i32, i32, u32, u32),
	renderPositionBottom: (i32, i32, u32, u32),
	position: Vector,
	idle: bool,
	hitbox: (i32, i32, u32, u32),
	iframeCounter: u32,
	health: i32,
	deathCounter: Option<DeathCounter>,
}

impl InnerSkeleton {
	pub fn fromSkeleton(skeleton: &Skeleton) -> InnerSkeleton {
		let &Skeleton {id, timer, renderPositionTop, renderPositionBottom, position, idle, hitbox, iframeCounter, health, deathCounter, ..} = skeleton;
		InnerSkeleton {id:id.getID(), timer, renderPositionTop:renderPositionTop.into(), renderPositionBottom: renderPositionBottom.into(), position, idle, hitbox:hitbox.into(), iframeCounter, health, deathCounter}
	}
}

#[derive(Debug)]
pub struct Skeleton<'a> {
	id: TypedID<'a, Self>,
	animationsTop: Animations<'a>,
	animationsBottom: Animations<'a>,
	timer: u32,
	renderPositionTop: Rect,
	renderPositionBottom: Rect,
	position: Vector,
	idle: bool,
	hitbox: Rect,
	iframeCounter: u32,
	health: i32,
	deathCounter: Option<DeathCounter>,
}
#[derive(Debug)]
pub struct SkeletonData {
	nextPos: Vector,
}

impl SkeletonData {
	fn doCollision(&mut self, skeleton: &Skeleton, ctx: &GameContext) {
		let mut hitbox = skeleton.hitbox;
		hitbox.reposition(self.nextPos);
		let map = ctx.getMap();
		let mut iter = map.calculateCollisionBounds(hitbox);
		while let Some((location, tile)) = map.collide(&mut iter) {
			match tile.getCollisionType() {
				CollisionType::Block => {
					self.nextPos += MapMod::blockCollide(location, hitbox, map);
					hitbox.reposition(self.nextPos);
				},
				_ => (),
			}
		}
	}
}

impl<'a> Skeleton<'a> {
	pub fn new(creator: &'a TextureCreator<WindowContext>, position: (f32, f32)) -> io::Result<BoxCode<'a>> {
		let (timer, position, idle, iframeCounter, health, deathCounter) = (
			0u32,
			Vector(position.0, position.1),
			false,
			0,
			10,
			None,
		);

		let animationsTop = Animations::new("Resources/Images/Skeleton_top.anim", NAMES_TOP, creator)?;
		let animationsBottom = Animations::new("Resources/Images/Skeleton_bottom.anim", NAMES_BOTTOM, creator)?;
		let renderPositionTop = Rect::new(position.0.round() as i32, position.1.round() as i32, 50, 50);
		let renderPositionBottom = Rect::new(position.0.round() as i32, position.1.round() as i32 + 50, 50, 50);
		let hitbox = Rect::new(position.0.round() as i32, position.1.round() as i32, 50, 100);
		Ok(BoxCode::Skeleton(
			Entity::new(
				Skeleton {id: TypedID::new(ID::empty()), animationsTop, animationsBottom, timer, renderPositionTop, renderPositionBottom, position, idle, hitbox, iframeCounter, health,deathCounter,},
				SkeletonData{
					nextPos: Vector(0f32, 0f32),
				},
			)
		))
	}
	pub fn fromInner(inner: InnerSkeleton, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Skeleton(
			Entity::new(
				Skeleton {
					id: TypedID::new(inner.id),
					animationsTop: Animations::new("Resources/Images/Skeleton_top.anim", NAMES_TOP, creator)?,
					animationsBottom: Animations::new("Resources/Images/Skeleton_bottom.anim", NAMES_BOTTOM, creator)?,
					timer: inner.timer,
					renderPositionTop: Rect::from(inner.renderPositionTop),
					renderPositionBottom: Rect::from(inner.renderPositionBottom),
					position: inner.position,
					idle: inner.idle,
					hitbox: Rect::from(inner.hitbox),
					iframeCounter: inner.iframeCounter,
					health: inner.health,
					deathCounter: inner.deathCounter,
				},
				SkeletonData {
					nextPos: Vector(0f32, 0f32),
				}
			)
		))
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.hitbox.has_intersection(hitbox)
	}
	fn updatePositions(&mut self, po: &mut PO) {
		self.renderPositionTop.reposition(self.position);
		self.renderPositionBottom.reposition(self.position + Vector(0f32, 50f32));
		let prevHitbox = self.hitbox;
		self.hitbox.reposition(self.position);
		po.updatePosition(self.id.getID(), self.hitbox, prevHitbox);
	//	println!("Skeleton: {:?}", self.hitbox);
	}
}

impl<'a> Collision for Skeleton<'a> {
	fn collide(&mut self, msg: Envelope<CollisionMsg>, po: &PO) {
		match msg.getMsg() {
			CollisionMsg::Damage(damage) => {
				if self.iframeCounter == 0 {
					self.health -= damage;
					self.iframeCounter = 90;
					self.animationsBottom.changeAnimation(ANIMATION_IDX_BOTTOM::Idle as usize);
					self.idle = true;
				}
			},
		}
	}
	fn collideWith(&self, other: ID, po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {
		if other == po.getCtx().getPlayerID().getID() {
			(Some(Envelope::new(CollisionMsg::Damage(8), other, self.id.getID())), key)
		}
		else {(None, key)}
	}
}

impl<'a> Counter for Skeleton<'a> {}

impl<'a> RegisterID for Skeleton<'a> {
	fn register(&mut self, id: IDRegistration) {
		match id {
			IDRegistration::DeathCounter(id) => self.deathCounter = Some(DeathCounter::new(id)),
		};
	}
}

impl<'a> EntityTraitsWrappable<'a> for Skeleton<'a> {
	type Data = SkeletonData;
	fn setID(&mut self, id: TypedID<'a, Self>) {
		self.id = id;
	}
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self> {
		if let RefCodeMut::Skeleton(s) = code {Some(s as &mut Self)}
		else {None}
	}
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self> {
		if let RefCode::Skeleton(s) = code {Some(s as &Self)}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key {
		if !self.idle {
			let player = po.getCtx().getHolder().getTyped(po.getCtx().getPlayerID()).unwrap();
			let playerDirection = Vector::fromPoints(self.position, player.getPosition());
			data.nextPos = self.position + playerDirection.normalizeOrZero() * 3.5f32;
			data.doCollision(self, po.getCtx());
		}
		else {
			data.nextPos = self.position;
		}
		key

	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		
		if self.health <= 0 {
			po.addToPurgeList(self.id.getID());
			po.removeCollision(self.id.getID(), self.hitbox);
			if let Some(counter) = self.deathCounter {
				counter.inc(Some(self.id.getID()), -1, po);
			}
			return;
		}

		self.position = data.nextPos;
		self.updatePositions(po);

		if !self.idle {
			self.timer += 1;
			if self.timer == 10 {
				self.animationsBottom.changeAnimation(ANIMATION_IDX_BOTTOM::Walk1 as usize);
			}
			if self.timer > 20 {
				self.timer = 0;
				self.animationsBottom.changeAnimation(ANIMATION_IDX_BOTTOM::Walk0 as usize);
			}
		}
		if self.iframeCounter > 0 {self.iframeCounter -= 1;}
		self.idle = self.iframeCounter != 0;
			
	}
	fn needsExecution(&self) -> bool {true}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		if self.iframeCounter / 10 % 2 != 1 {
			self.animationsTop.drawNextFrame(canvas, self.renderPositionTop);
			self.animationsBottom.drawNextFrame(canvas, self.renderPositionBottom);
		}
	}
}

