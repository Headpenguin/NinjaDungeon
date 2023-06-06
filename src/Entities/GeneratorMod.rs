use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use serde::{Serialize, Deserialize};

use std::io;

use crate::{Tile, ID};
use crate::EventProcessor::{Envelope, CollisionMsg, PO, Key};
use crate::Entities::{TypedID, BoxCode, RefCode, RefCodeMut};
use crate::Entities::Traits::{Collision, EntityTraitsWrappable, Entity};
use crate::SpriteLoader::Sprites;

const NAME: &'static[&'static str] = &["Resources/Images/Generator.png"];

#[derive(Serialize, Deserialize)]
pub struct InnerGenerator {
	renderRect: (i32, i32, u32, u32),
	tiles: Vec<(Tile, (u16, u16))>,
}

impl InnerGenerator {
	pub fn fromGenerator(Generator { renderRect, tiles, .. }: &Generator) -> InnerGenerator {
		InnerGenerator {
			renderRect: (*renderRect).into(),
			tiles: tiles.clone(),
		}
	}
}
#[derive(Debug)]
pub struct Generator<'a> {
	renderRect: Rect,
	tiles: Vec<(Tile, (u16, u16))>,
	editor: bool,
	sprite: Sprites<'a>,
}

impl<'a> Generator<'a> {
	pub fn new(creator: &'a TextureCreator<WindowContext>, pos: (i32, i32), tiles: Vec<(Tile, (u16, u16))>) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Generator(
			Entity::new(
				Generator {
					renderRect: Rect::new(pos.0, pos.1, 50, 50),
					tiles,
					editor: true,
					sprite: Sprites::new(creator, NAME)?
				},
				()
			)
		))
	}
	pub fn fromInner(InnerGenerator { renderRect, tiles }: InnerGenerator, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		Ok(BoxCode::Generator(
			Entity::new(
				Generator {
					renderRect: Rect::from(renderRect),
					tiles,
					editor: true,
					sprite: Sprites::new(creator, NAME)?,
				},
				()
			)
		))
	}
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		self.renderRect.has_intersection(hitbox)
	}
}

impl<'a> Collision for Generator<'a> {
	fn collideWith(&self, _other: ID, _po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {(None, key)}
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

