use serde::{Serialize, Deserialize};

use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use std::io;
use std::cmp::Ordering;

use super::Traits::{Collision, EntityTraitsWrappable, Entity, Counter, RegisterID, IDRegistration};
use super::{BoxCode, RefCode, RefCodeMut, TypedID};
use super::Common::DeathCounter;
use crate::SpriteLoader::Animations;
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
pub struct Cannon<'a> {
	id: TypedID<'a, Self>,
	animations: Animations<'a>,
	pos: Vector,
	hitbox: Rect,
	renderPosition: Rect,
	timer: u16,
	variant: u8,
	deathEvent: Option<DeathCounter>,
}

#[derive(Debug, Default)]
pub struct CannonData {

}

impl<'a> Cannon<'a> {
	fn newInt(creator: &'a TextureCreator<WindowContext>, pos: Vector, variant: u8, deathEvent: Option<DeathCounter>) -> io::Result<Self> {
		Ok(Cannon {
			id: TypedID::new(ID::empty()),
			animations: Animations::new("Resources/Images/Cannon.anim", NAMES, creator)?,
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
	fn checkLineOfSight(pos: Vector, line: Vector, po: &PO) -> bool {
		let m = line.1 / line.0;
		let (startx, endx) = if line.0 > 0f32 {
			(pos.0, line.0 + pos.0)
		} else {
			(line.0 + pos.0, pos.0)
		};
		let (starty, endy) = if line.0 > 0f32 {
			(pos.1, pos.1 + line.1)
		} else {
			(pos.1 + line.1, pos.1)
		};
		for x in ((startx as i32 / 50) as u16)..=((endx as i32 / 50) as u16) {
			let (y0, y1) = {
				let x = x as f32 * 50f32;
				let x1 = x as f32 + 50f32;
				let y = (x - startx) * m + starty;
				let y1 = (x1 - startx) * m + starty;
				(y, y1)
			};
			let (y0, y1) = if y0 > y1 {(y1, y0)} else {(y0, y1)};
			let (starty, endy) = if starty > endy {(endy, starty)} else {(starty, endy)};
			let (y0, y1) = ((std::cmp::max_by(y0, starty, |a, b| a.partial_cmp(b).unwrap_or(Ordering::Less)) / 50f32) as u16, (std::cmp::min_by(y1, endy, |a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater)) / 50f32) as u16);
			for y in y0..=y1 {
				match po.getCtx().getMap().getScreen(po.getCtx().getMap().getActiveScreenId()).unwrap().getTile((x, y)).getCollisionType() {
					CollisionType::Block => {
						println!("{:?}", pos);
						return false;
					}
					CollisionType::OOB => {},
					_ => ()
				};
			}
			
		}
		true
	}
}

impl<'a> Collision for Cannon<'a> {
	fn collide(&mut self, msg: Envelope<CollisionMsg>, po: &PO) {
		unimplemented!()
	}
	fn collideWith(&self, other: ID, po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {
		unimplemented!()
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
		let playerPos = po.getCtx().getHolder().getTyped(po.getCtx().getPlayerID()).unwrap().getCenter();
		if Self::checkLineOfSight(self.pos, playerPos - self.pos, po) {
		}
		key
	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
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
	}
}

