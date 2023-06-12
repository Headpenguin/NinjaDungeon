use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::{Rect, Point};

use serde::{Serialize, Deserialize};

use std::io;

use crate::{Tile, ID, GameContext, Vector};
use crate::EventProcessor::{Envelope, CollisionMsg, CounterMsg, PO, Key};
use crate::Entities::{TypedID, BoxCode, RefCode, RefCodeMut, EntityBuilder};
use crate::Entities::Traits::{Collision, RegisterID, EntityTraitsWrappable, Entity, Counter};
use crate::SpriteLoader::Sprites;
use crate::MapMod::CollisionType;

use std::f32::consts;

const NAMES: &'static [&'static str] = &["Resources/Images/SnakeBossHead.png", "Resources/Images/SnakeBossTail.png"];

const SCREEN_CENTER: Vector = Vector(17.0 * 25.0, 12.0 * 25.0);

#[derive(Serialize, Deserialize)]
pub struct InnerSnakeBoss {
	id: ID,
}

impl InnerSnakeBoss {
	pub fn fromSnakeBoss(&SnakeBoss {id, ..} : &SnakeBoss) -> Self {
		InnerSnakeBoss {
			id: id.getID(),
		}
	}
}

#[derive(Debug)]
pub struct SnakeBoss<'a> {
	id: TypedID<'a, Self>,
	sprites: Sprites<'a>,
	angleStart: f32,
	angleEnd: f32,
	playerInformed: bool,
	activated: bool,
}

#[derive(Debug, Default)]
pub struct SnakeBossData {
	activate: bool,
}

impl<'a> SnakeBoss<'a> {
	fn newInt(creator: &'a TextureCreator<WindowContext>) -> io::Result<Self> {
		Ok(SnakeBoss {
			id: TypedID::new(ID::empty()),
			sprites: Sprites::new(creator, NAMES)?,
			angleStart: consts::PI * (2.0 - 0.25),
			angleEnd: consts::PI * (2.0 - 0.75),
			activated: false,
			playerInformed: false,
		})
	}
	pub fn new(creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::SnakeBoss(
			Entity::new(
				Self::newInt(creator)?,
				SnakeBossData::default()
			)
		))
	}
	pub fn fromInner(InnerSnakeBoss { id }: InnerSnakeBoss, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		let mut tmp = Self::newInt(creator)?;
		tmp.setID(TypedID::new(id));
		Ok(BoxCode::SnakeBoss(
			Entity::new(
				tmp,
				SnakeBossData::default()
			)
		))
	}
	pub fn collidesStatic(&self, _hitbox: Rect) -> bool {false}
	fn dropGates(&self, po: &PO) {
		for i in 2..=14 {
			po.spawnTile(Tile::gate(), (i, 11));
		}
	}
	fn getPos(&self, angle: f32) -> Vector {
		Vector(4.5 * 50.0 * angle.cos(), -3.0 * 50.0 * angle.sin())
	}
	fn calcDrawInfo(&self, inAngle: f32) -> (Rect, f32) {
		let center = self.getPos(inAngle);
		let mut angle = (center.1 / center.0).atan() - consts::PI / 2.0; 
		if inAngle > consts::PI / 2.0 && inAngle < 3.0 * consts::PI / 2.0 {
			angle += consts::PI;
		}
		let center = center + SCREEN_CENTER;
		let render = Rect::from_center(<Vector as Into<(i32, i32)>>::into(center), 50, 250);
		(render, angle)
	}
	pub fn collides(&self, mut pos: Vector) -> bool {
		pos -= SCREEN_CENTER;
		let mut angle = (pos.1 / pos.0).atan();
		if pos.0 < 0.0 {
			angle += consts::PI;
		}
		if angle < 0.0 {
			angle += consts::PI * 2.0;
		}
		let (_, mut angleStart) = self.calcDrawInfo(self.angleStart);
		let (_, mut angleEnd) = self.calcDrawInfo(self.angleEnd);
		angleStart += consts::PI / 2.0;
		angleEnd += consts::PI / 2.0;
		if angleStart < 0.0 {angleStart += consts::PI * 2.0;}
		if angleEnd < 0.0 {angleEnd += consts::PI * 2.0;}
		if angleStart > angleEnd {
			angle > angleEnd && angle < angleStart
		}
		else {
			angle < angleStart || angle > angleEnd
		}
	}
	fn die(&mut self, po: &PO) {
		for angle in 0..24 {

			let (x, y): (i32, i32) = ((self.getPos(angle as f32 * consts::PI * 2.0 / 24.0) + SCREEN_CENTER) / 50.0).into();
			
			for x in (x-1)..=(x+1) {
				for y in (y-1)..=(y+1) {
					po.spawnTile(Tile::new(0, CollisionType::None), (x as u16, y as u16));
				}
			}

		}
		po.spawnTile(Tile::new(0, CollisionType::None), (10, 5));
		po.spawnTile(Tile::new(0, CollisionType::None), (10, 7));
		po.spawnTile(Tile::new(0, CollisionType::Transition(17)), (16, 7));
		po.spawnTile(Tile::new(0, CollisionType::Transition(17)), (16, 6));
		po.spawnTile(Tile::new(0, CollisionType::Transition(17)), (16, 8));
		po.informPlayerSnakeBossDeath();
		po.addToPurgeList(self.id.getID());
		
	}
}

impl<'a> Collision for SnakeBoss<'a> {
	fn collideWith(&self, _id: ID, _other: ID, _po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {(None, key)}
}

impl<'a> RegisterID for SnakeBoss<'a> {}
impl<'a> Counter for SnakeBoss<'a> {}

impl<'a> EntityTraitsWrappable<'a> for SnakeBoss<'a> {
	type Data = SnakeBossData;
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self> {
		if let RefCodeMut::SnakeBoss(s) = code {
			Some(s)
		}
		else {None}
	}
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self> {
		if let RefCode::SnakeBoss(s) = code {
			Some(s)
		}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key {
		data.activate = false;
		let player = po.getCtx().getHolder().getTyped(po.getCtx().getPlayerID()).unwrap();
		if self.playerInformed && !self.activated {
			data.activate = player.isActivateSnakeBoss();
		}
		key
	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		if !self.playerInformed {
			po.informPlayerSnakeBoss(self.id.getID());
			self.playerInformed = true;
		}
		if !self.activated && data.activate {
			self.dropGates(po);
			self.activated = true;
		}
		if !self.activated {return;}
		
		self.angleStart -= 0.015;
		if self.angleStart <= 0f32 {
			self.angleStart += consts::PI * 2.0;
		}
		
		self.angleEnd -= 0.015;
		if self.angleEnd <= 0f32 {
			self.angleEnd += consts::PI * 2.0;
		}

		let (x, y): (i32, i32) = ((self.getPos(self.angleStart + 0.45) + SCREEN_CENTER) / 50.0).into();
		for x in (x-1)..=(x+1) {
			for y in (y-1)..=(y+1) {
				if let CollisionType::SnakeKill = po.getCtx().getMap().getScreen(po.getCtx().getMap().getActiveScreenId()).unwrap().getTile((x as u16, y as u16)).getCollisionType() {
					self.die(po);
					return;
				}
				po.spawnTile(Tile::new(19, CollisionType::None), (x as u16, y as u16));
			}
		}
		
		let (x, y): (i32, i32) = ((self.getPos(self.angleEnd + 0.1) + SCREEN_CENTER) / 50.0).into();
		
		for x in (x-1)..=(x+1) {
			for y in (y-1)..=(y+1) {
				if let CollisionType::SnakeKill = po.getCtx().getMap().getScreen(po.getCtx().getMap().getActiveScreenId()).unwrap().getTile((x as u16, y as u16)).getCollisionType() {
					self.die(po);
					return;
				}
				po.spawnTile(Tile::new(0, CollisionType::None), (x as u16, y as u16));
			}
		}

		po.spawnTile(Tile::new(1, CollisionType::Block), (7, 5));
	
	}
	fn needsExecution(&self) -> bool {true}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		let (mut render, mut angle) = self.calcDrawInfo(self.angleStart);
		angle *= 180.0 / consts::PI;
        //let center = render.center();
        render.resize(100, 250);
		self.sprites.getSprite(0).drawRot(canvas, render, angle as f64, Point::new(25, 125));
		let (mut render, mut angle) = self.calcDrawInfo(self.angleEnd);
		angle *= 180.0 / consts::PI;
        //let center = render.center();
        render.resize(100, 250);
		self.sprites.getSprite(1).drawRot(canvas, render, angle as f64, Point::new(75, 125));
	}
	fn setID(&mut self, id: TypedID<'a, Self>) {self.id = id;}
}

