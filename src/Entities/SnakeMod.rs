use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use serde::{Serialize, Deserialize};

use std::io;

use crate::{Tile, ID, GameContext, Vector, Direction};
use crate::EventProcessor::{Envelope, CollisionMsg, PO, Key};
use crate::Entities::{TypedID, BoxCode, RefCode, RefCodeMut, EntityBuilder};
use crate::Entities::Traits::{Collision, RegisterID, EntityTraitsWrappable, Entity, Counter};
use crate::SpriteLoader::Sprites;

const NAMES: &'static [&'static str] = &[
	"Resources/Images/SnakeHead.png",
	"Resources/Images/SnakeHeadLeft.png",
	"Resources/Images/SnakeHeadUp.png",
	"Resources/Images/SnakeHeadDown.png",
];

#[derive(Serialize, Deserialize)]
pub struct InnerSnake {
	id: ID,
	dir: Direction,
	pos: (u16, u16),
}

impl InnerSnake {
	pub fn fromSnake(&Snake { id, dir, pos, ..}: &Snake) -> InnerSnake {
		InnerSnake {id: id.getID(), dir, pos}
	}
}

#[derive(Debug)]
pub struct Snake<'a> {
	id: TypedID<'a, Self>,
	dir: Direction,
	pos: (u16, u16),
	editorRender: Rect,
	editorSprite: Sprites<'a>,
	timer: u16,
}

#[derive(Debug)]
pub struct SnakeData {}

impl<'a> Snake<'a> {
	fn newInt(creator: &'a TextureCreator<WindowContext>, pos: (u16, u16), dir: Direction) -> io::Result<Self> {
		Ok(Snake {
			id: TypedID::new(ID::empty()),
			dir,
			pos,
			editorRender: Rect::new(pos.0 as i32 * 50, pos.1 as i32 * 50, 50, 50),
			editorSprite: Sprites::new(creator, NAMES)?,
			timer: u16::MAX,
		})
	}
	pub fn new(creator: &'a TextureCreator<WindowContext>, pos: (u16, u16), dir: Direction) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Snake(
			Entity::new(
				Self::newInt(creator, pos, dir)?,
				SnakeData {},
			)
		))
	}
	pub fn fromInner(InnerSnake { id, dir, pos }: InnerSnake, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		let mut tmpSnake = Snake::newInt(creator, pos, dir)?;
		tmpSnake.setID(TypedID::new(id));
		Ok(BoxCode::Snake(
			Entity::new(
				tmpSnake,
				SnakeData {},
			)
		))
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.editorRender.has_intersection(hitbox)
	}
}

impl<'a> Collision for Snake<'a> {
	fn collideWith(&self, _other: ID, _po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {(None, key)}
}
impl<'a> RegisterID for Snake<'a> {}
impl<'a> Counter for Snake<'a> {}

impl<'a> EntityTraitsWrappable<'a> for Snake<'a> {
	type Data = SnakeData;
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self> {
		if let RefCodeMut::Snake(snake) = code {
			Some(snake)
		}
		else {None}
	}
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self> {
		if let RefCode::Snake(snake) = code {
			Some(snake)
		}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key {
		key
	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		
	}
	fn needsExecution(&self) -> bool {
		self.timer == 59
	}
	fn tick(&mut self) {
		if self.timer == u16::MAX {self.timer = 0}
		else {
			self.timer += 1;
			if self.timer == 60 {self.timer = 0;}
		}
	}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		if self.timer == u16::MAX {
			let idx = match self.dir {
				Direction::Right => 0,
				Direction::Left => 1,
				Direction::Up => 2,
				Direction::Down => 3,
			};
			self.editorSprite.getSprite(idx).draw(canvas, self.editorRender, false, false);
		}
	}
	fn setID(&mut self, id: TypedID<'a, Self>) {self.id = id;}
}

