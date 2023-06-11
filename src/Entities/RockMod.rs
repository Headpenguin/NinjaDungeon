use serde::{Serialize, Deserialize};

use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use std::io;

use super::Traits::{Collision, EntityTraitsWrappable, Entity, Counter, RegisterID};
use super::{BoxCode, RefCode, RefCodeMut, TypedID};
use crate::SpriteLoader::Animations;
use crate::{GameContext, Vector, ID, Direction};
use crate::EventProcessor::{CollisionMsg, Envelope, PO, Key};
use crate::CollisionType;
use crate::MapMod;

const NAMES: &'static[&'static str] = &[
	"Walk",
	"WalkLeft",
];

#[derive(Serialize, Deserialize)]
pub struct InnerRock {
	id: ID,
	path: Vec<(u16, u16)>,
}

impl InnerRock {
	pub fn fromRock(Rock {id, path, ..}: &Rock) -> Self {
		InnerRock {id: id.getID(), path: path.clone()}
	}
}

#[derive(Debug)]
pub struct Rock<'a> {
	id: TypedID<'a, Self>,
	animations: Animations<'a>,
	timer: u16,
	position: Vector,
	dp: Vector,
	hitbox: Rect,
	renderPosition: Rect,
	path: Vec<(u16, u16)>,
	currentPath: usize,
	dir: Direction,
}

impl<'a> Rock<'a> {
	pub fn newInt(creator: &'a TextureCreator<WindowContext>, path: Vec<(u16, u16)>) -> io::Result<Self> {
		let position = path[0];
		let position = Vector(position.0 as f32, position.1 as f32);
		let dir = Self::determineDirection(&path, 0);
		let dp = Self::getDp(&path, 0);
		Ok(Rock {
			id: TypedID::new(ID::empty()),
			animations: Animations::new("Resources/Images/Rock.anim", NAMES, creator)?,
			timer: 0,
			position,
			dp,
			hitbox: Rect::new(position.0 as i32 * 50, position.1 as i32 * 50, 50, 50),
			renderPosition: Rect::new(position.0 as i32 * 50, position.1 as i32 * 50, 50, 50),
			path,
			currentPath: 0,
			dir,
		})
	}
	pub fn new(creator: &'a TextureCreator<WindowContext>, path: Vec<(u16, u16)>) -> io::Result<BoxCode<'a>> {
		Ok(
			BoxCode::Rock(
				Entity::new(
					Self::newInt(creator, path)?,
					()
				)
			)
		)
	}
	pub fn fromInner(InnerRock {id, path} : InnerRock, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		let mut rock = Self::newInt(creator, path)?;
		rock.setID(TypedID::new(id));
		Ok(BoxCode::Rock(
			Entity::new(
				rock,
				()
			)
		))
	}
	fn getDp(path: &[(u16, u16)], currPath: usize) -> Vector {
		let (x1, y1) = path[currPath];
		let (x2, y2) = path[(currPath + 1) % path.len()];
		let pos1 = (x1 as i32 * 50, y1 as i32 * 50);
		let pos2 = (x2 as i32 * 50, y2 as i32 * 50);
		Vector::fromPoints(pos1, pos2) / 20f32
	}
	fn determineDirection(path: &[(u16, u16)], currPath: usize) -> Direction {
		let (x1, y1) = path[currPath];
		let (x2, y2) = path[(currPath + 1) % path.len()];
		let info = (x2 as i32 - x1 as i32 > 0, y2 as i32 - y1 as i32 > 0, (x2 as i32 - x1 as i32).abs() > (y2 as i32 - y1 as i32).abs());
		if info.2 {
			if info.0 {
				Direction::Right
			}
			else {
				Direction::Left
			}
		}
		else {
			if info.1 {
				Direction::Down
			}
			else {
				Direction::Up
			}
		}
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.hitbox.has_intersection(hitbox)
	}
	fn updatePositions(&mut self, po: &mut PO) {
		let oldPos = self.hitbox;
		self.hitbox = Rect::new(self.position.0 as i32, self.position.1 as i32, 50, 50);
		self.renderPosition = self.hitbox;
		self.dp = Self::getDp(&self.path, self.currentPath);
		po.updatePosition(self.id.getID(), oldPos, self.hitbox);
	}
}

impl<'a> Collision for Rock<'a> {
	fn collideWith(&self, other: ID, po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {
		(Some(Envelope::new(CollisionMsg::Ground(self.hitbox, Self::getDp(&self.path, self.currentPath)), other, self.id.getID())), key)
	}
}

impl<'a> Counter for Rock<'a> {}
impl<'a> RegisterID for Rock<'a> {}

impl<'a> EntityTraitsWrappable<'a> for Rock<'a> {
	type Data = ();
	fn setID(&mut self, id: TypedID<'a, Self>) {
		self.id = id;
	}
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self> {
		if let RefCodeMut::Rock(rock) = code {Some(rock as &mut Self)}
		else {None}
	}
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self> {
		if let RefCode::Rock(rock) = code {Some(rock as &Self)}
		else {None}
	}
	fn getData(&self, _data: &mut Self::Data, po: &PO, key: Key) -> Key {
		for entity in po.getCtx().getCollisionList(self.id.getID()) {
			po.sendCollisionMsg(Envelope::new(CollisionMsg::Ground(self.hitbox, self.dp), entity, self.id.getID()));
		}
		key
	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		let (x1, y1) = self.path[self.currentPath];
		let (x2, y2) = self.path[(self.currentPath + 1) % self.path.len()];
		self.position = Vector::fromPoints((x1 as i32, y1 as i32), (x2 as i32, y2 as i32)) * 50f32 * self.timer as f32 / 20f32 + Vector::from((x1 as i32, y1 as i32)) * 50f32;
		self.updatePositions(po);
		self.timer += 1;
		if self.timer > 20 {
			self.currentPath += 1;
			self.currentPath %= self.path.len();
			self.dir = Self::determineDirection(&self.path, self.currentPath);
			self.timer = 0;
		}
		if self.timer % 10 == 1 {
			self.animations.update();
		}
		match self.dir {
			Direction::Up | Direction::Right => self.animations.changeAnimation(0),
			_ => self.animations.changeAnimation(1),
		};
	}
	fn needsExecution(&self) -> bool {true}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		self.animations.drawNextFrame(canvas, self.renderPosition);
	}
}

