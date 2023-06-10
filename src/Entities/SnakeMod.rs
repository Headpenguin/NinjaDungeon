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
use crate::MapMod::CollisionType;

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

#[derive(Debug, Default)]
pub struct SnakeData {
	dir: Direction,
	pos: (u16, u16),
}

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
				SnakeData::default(),
			)
		))
	}
	pub fn fromInner(InnerSnake { id, dir, pos }: InnerSnake, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		let mut tmpSnake = Snake::newInt(creator, pos, dir)?;
		tmpSnake.setID(TypedID::new(id));
		Ok(BoxCode::Snake(
			Entity::new(
				tmpSnake,
				SnakeData::default(),
			)
		))
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.editorRender.has_intersection(hitbox)
	}

	fn nextPos(mut pos: (u16, u16), dir: Direction) -> (u16, u16) {
		match dir {
			Direction::Up => pos.1 -= 1,
			Direction::Down => pos.1 += 1,
			Direction::Left => pos.0 -= 1,
			Direction::Right => pos.0 += 1,
		}
		pos
	}
	fn determineMovementDirection(toPlayer: Vector, pos: (u16, u16), po: &PO) -> Option<Direction> {
		let dirInfo = (toPlayer.0.abs() > toPlayer.1.abs(), toPlayer.0 > 0f32, toPlayer.1 > 0f32);
		let order = match dirInfo {
			(true, true, true) => [Direction::Right, Direction::Down, Direction::Up, Direction::Left],
			(true, true, false) => [Direction::Right, Direction::Up, Direction::Down, Direction::Left],
			(true, false, true) => [Direction::Left, Direction::Up, Direction::Down, Direction::Right],
			(true, false, false) => [Direction::Left, Direction::Down, Direction::Up, Direction::Right],
			(false, true, true) => [Direction::Down, Direction::Right, Direction::Left, Direction::Up],
			(false, true, false) => [Direction::Up, Direction::Right, Direction::Left, Direction::Down],
			(false, false, true) => [Direction::Down, Direction::Left, Direction::Right, Direction::Up],
			(false, false, false) => [Direction::Up, Direction::Left, Direction::Right, Direction::Down],
		};
		for direction in order {
			let tile = po.getCtx().getMap().getScreen(po.getCtx().getMap().getActiveScreenId()).unwrap().getTile(Self::nextPos(pos, direction));
			match tile.getCollisionType() {
				CollisionType::Block | CollisionType::Hit(..) => continue,
				_ => return Some(direction),
			}
		}
		None
	}
	fn snakeTile(dir1: Direction, dir2: Direction) -> Tile {
		let offset = match (dir1, dir2) {
			(Direction::Left, Direction::Right) | (Direction::Right, Direction::Left) | (Direction::Left, Direction::Left) | (Direction::Right, Direction::Right) => 0,
			(Direction::Up, Direction::Down) | (Direction::Down, Direction::Up) | (Direction::Up, Direction::Up) | (Direction::Down, Direction::Down) => 1,
			(Direction::Right, Direction::Up) | (Direction::Down, Direction::Left) => 2,
			(Direction::Right, Direction::Down) | (Direction::Up, Direction::Left) => 3,
			(Direction::Left, Direction::Up) | (Direction::Down, Direction::Right) => 4,
			(Direction::Left, Direction::Down) | (Direction::Up, Direction::Right) => 5,
		};
		Tile::new(9 + offset, CollisionType::Hit(-12))
	}
	fn snakeHeadTile(dir: Direction) -> Tile {
		let offset = match dir {
			Direction::Right => 0,
			Direction::Left => 1,
			Direction::Up => 2,
			Direction::Down => 3,
		};
		Tile::new(5 + offset, CollisionType::Hit(-12))
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
		data.pos = self.pos;
		data.dir = self.dir;
		let playerPos = po.getCtx().getHolder().getTyped(po.getCtx().getPlayerID()).unwrap().getPosition();
		let pos = Vector(self.pos.0 as f32 * 50f32, self.pos.1 as f32 * 50f32);
		let toPlayer = playerPos - pos;
		let dir = Self::determineMovementDirection(toPlayer, self.pos, po);
		if let Some(dir) = dir {
			data.pos = Self::nextPos(self.pos, dir);
			po.spawnTile(Self::snakeTile(self.dir, dir), self.pos);
			po.spawnTile(Self::snakeHeadTile(dir), data.pos);
			data.dir = dir;
		}
		key
	}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		self.dir = data.dir;
		self.pos = data.pos;
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

