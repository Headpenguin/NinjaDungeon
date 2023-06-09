use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use serde::{Serialize, Deserialize};

use std::io;

use crate::{Tile, ID};
use crate::EventProcessor::{Envelope, CollisionMsg, CounterMsg, PO, Key};
use crate::Entities::{TypedID, BoxCode, RefCode, RefCodeMut};
use crate::Entities::Traits::{Collision, RegisterID, EntityTraitsWrappable, Entity, Counter};
use crate::SpriteLoader::Sprites;

const NAME: &'static[&'static str] = &["Resources/Images/Generator.png"];

#[derive(Serialize, Deserialize)]
pub struct InnerGenerator {
	renderRect: (i32, i32, u32, u32),
	tiles: Vec<(Tile, (u16, u16))>,
	cnt: u8,
}

#[derive(Serialize, Deserialize)]
pub struct InnerEntityGenerator {
    gen: InnerGenerator,
    entities: Vec<(ID, bool)>,
}

impl InnerGenerator {
	pub fn fromGeneratorInt(Generator { renderRect, tiles, cnt, .. }: &Generator) -> InnerGenerator {
		InnerGenerator {
			renderRect: (*renderRect).into(),
			tiles: tiles.clone(),
			cnt: *cnt,
		}
    }
	pub fn fromGenerator(gen: &Generator) -> InnerGenerator {
	    fromGeneratorInt(gen)
    }
}
impl InnerEntityGenerator {
	pub fn fromEntityGenerator(EntityGenerator { gen, entities }: &EntityGenerator) -> InnerEntityGenerator {
        InnerEntityGenerator {
            gen: InnerGenerator::fromGeneratorInt(gen),
            entities,
        }
	}
}
#[derive(Debug)]
pub struct Generator<'a> {
	renderRect: Rect,
	tiles: Vec<(Tile, (u16, u16))>,
	editor: bool,
	sprite: Sprites<'a>,
	cnt: u8,
}

#[derive(Debug)]
pub struct EntityGenerator<'a> {
    gen: Generator<'a>,
    entities: Vec<(ID, bool)>,
}

impl<'a> Generator<'a> {
	pub fn newInt(creator: &'a TextureCreator<WindowContext>, pos: (i32, i32), tiles: Vec<(Tile, (u16, u16))>, cnt: u8) -> io::Result<Self> {
        Ok(Generator {
            renderRect: Rect::new(pos.0, pos.1, 50, 50),
            tiles,
            editor: true,
            sprite: Sprites::new(creator, NAME)?,
            cnt,
        })
    }
	pub fn new(creator: &'a TextureCreator<WindowContext>, pos: (i32, i32), tiles: Vec<(Tile, (u16, u16))>, cnt: u8) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Generator(
			Entity::new(
                Self::newInt(creator, pos, tiles, cnt)?,
				()
			)
		))
	}
	pub fn fromInnerInt(InnerGenerator { renderRect, tiles, cnt }: InnerGenerator, creator: &'a TextureCreator<WindowContext>) -> io::Result<Self> {
        Self::newInt(creator, renderRect.top_left(), tiles, cnt)
    }
	pub fn fromInner(InnerGenerator { renderRect, tiles, cnt }: InnerGenerator, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Generator(
			Entity::new(
                Self::newInt(creator, renderRect.top_left(), tiles, cnt)?,
				()
			)
		))
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.renderRect.has_intersection(hitbox)
	}
    fn activate(&mut self, po: &PO) {
		if self.cnt == 0 {
			for (tile, location) in self.tiles.drain(..) {
				po.spawnTile(tile, location);
			}
		}
    }
}

impl<'a> EntityGenerator<'a> {
	pub fn new(creator: &'a TextureCreator<WindowContext>, pos: (i32, i32), tiles: Vec<(Tile, (u16, u16))>, entities: Vec<(ID, bool)>, cnt: u8) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Generator(
			Entity::new(
                gen: Generator::newInt(creator, pos, tiles, cnt)?,
                entities,
				},
				()
			)
		))
	}
	pub fn fromInner(InnerEntityGenerator {gen, entities}, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Generator(
			Entity::new(
				EntityGenerator {
                    gen: Generator::fromInnerInt(gen)?,
                    entities,
				},
				()
			)
		))
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.gen.renderRect.has_intersection(hitbox)
	}
    fn activate(&mut self, po: &PO) {
        if self.cnt == 0 {
            for (entity, global) in self.entities.drain(..) {
                po.activateEntity(entity, global);
            }
        }
    }
}

impl<'a> Collision for Generator<'a> {
	fn collideWith(&self, _other: ID, _po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {(None, key)}
}
impl<'a> Collision for EntityGenerator<'a> {
	fn collideWith(&self, _other: ID, _po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {(None, key)}
}

impl<'a> RegisterID for Generator<'a> {}
impl<'a> RegisterID for EntityGenerator<'a> {}

impl<'a> Counter for Generator<'a> {
	fn inc(&mut self, msg: Envelope<CounterMsg>, po: &PO) {
		self.cnt = (self.cnt as i32 + msg.getMsg().0) as u8;
        self.activate(po);
	}
}
impl<'a> Counter for EntityGenerator<'a> {
	fn inc(&mut self, msg: Envelope<CounterMsg>, po: &PO) {
        self.gen.inc(msg, po);
        self.activate(po);
	}
}

impl<'a> EntityTraitsWrappable<'a> for Generator<'a> {
	type Data = ();
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self> {
		if let RefCodeMut::Generator(gen) = code {
			Some(gen)
		}
		else {None}
	}
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self> {
		if let RefCode::Generator(gen) = code {
			Some(gen)
		}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key {key}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		self.editor = false;
	}
	fn needsExecution(&self) -> bool {self.editor}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		if self.editor {
			self.sprite.getSprite(0).draw(canvas, self.renderRect, false, false);
		}
	}
	fn setID(&mut self, id: TypedID<'a, Self>) {}
}
impl<'a> EntityTraitsWrappable<'a> for EntityGenerator<'a> {
	type Data = ();
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self> {
		if let RefCodeMut::EntityGenerator(gen) = code {
			Some(gen)
		}
		else {None}
	}
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self> {
		if let RefCode::EntityGenerator(gen) = code {
			Some(gen)
		}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key {key}
	fn update(&mut self, data: &Self::Data, po: &mut PO) {
		self.editor = false;
	}
	fn needsExecution(&self) -> bool {self.editor}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		if self.editor {
			self.sprite.getSprite(0).draw(canvas, self.renderRect, false, false);
		}
	}
	fn setID(&mut self, id: TypedID<'a, Self>) {}
}

