use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{WindowContext, Window};
use sdl2::rect::Rect;

use super::Skeleton;
use crate::Player;
use super::BoxCode;
use crate::SpriteLoader::Sprites;
use crate::GameContext;
use std::io;

const ENTITY_SPRITES: &'static [&'static str] = &[
	"Resources/Images/Ninja_float_0__half.png",
	"Resources/Images/Skeleton_top__half.png",
];

pub struct EntityBuilder {
	id: u16,
	position: (u16, u16),
}

pub enum EntityBuilderSignals<'a> {
	Complete(io::Result<BoxCode<'a>>),
	InvalidId,
}

impl EntityBuilder {
	pub fn new(id: u16, position: (u16, u16)) -> EntityBuilder {
		EntityBuilder {
			id,
			position,
		}
	}
	pub fn build<'a>(&self, creator: &'a TextureCreator<WindowContext>) -> EntityBuilderSignals<'a> {
		match self.id {
			0 => EntityBuilderSignals::Complete(Player::new(creator, self.position.0 as f32 * 50f32, self.position.1 as f32 * 50f32)),
			1 => EntityBuilderSignals::Complete(Skeleton::new(creator, (self.position.0 as f32 * 50f32, self.position.1 as f32 * 50f32))),
			MAX_ENTITY_IDX.. => EntityBuilderSignals::InvalidId,
		}
	}
	pub fn addEntityGlobal<'a>(&self, ctx: &mut GameContext<'a>, entity: BoxCode<'a>) {
		match self.id {
			0 => ctx.addEntityGlobal::<Player>(entity),
			1 => ctx.addEntityGlobal::<Skeleton>(entity),
			MAX_ENTITY_IDX.. => unreachable!(),
		};
	}
	pub fn addEntityActiveScreen<'a>(&self, ctx: &mut GameContext<'a>, entity: BoxCode<'a>) {
		match self.id {
			0 => ctx.addEntityActiveScreen::<Player>(entity),
			1 => ctx.addEntityActiveScreen::<Skeleton>(entity),
			MAX_ENTITY_IDX.. => unreachable!(),
		};
	}
	pub fn getEntityRect(&self) -> Rect {
		let (w, h) = match self.id {
			0 => (50, 50),
			1 => (50, 100),
			MAX_ENTITY_IDX.. => unreachable!(),
		};
		Rect::new(self.position.0 as i32 * 50, self.position.1 as i32 * 50, w, h)
	}
}

pub struct EntityRenderer<'a> {
	entities: Sprites<'a>,
}

impl<'a> EntityRenderer<'a> {
	pub fn new(creator: &'a TextureCreator<WindowContext>) -> io::Result<Self> {
		Ok(EntityRenderer {
			entities: Sprites::new(creator, ENTITY_SPRITES)?,
		})
	}
	pub fn render(&self, canvas: &mut Canvas<Window>, id: u16, position: Rect) {
		self.entities.getSprite(id as usize).draw(canvas, position, false, false);
	}
}

pub const MAX_ENTITY_IDX: u16 = 1;

