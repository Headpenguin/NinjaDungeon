use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{WindowContext, Window};
use sdl2::rect::Rect;

use super::{Skeleton, Generator};
use crate::{Player, Tile};
use super::BoxCode;
use crate::SpriteLoader::Sprites;
use crate::GameContext;
use std::io;

const ENTITY_SPRITES: &'static [&'static str] = &[
	"Resources/Images/Ninja_float_0__half.png",
	"Resources/Images/Skeleton_top__half.png",
    "Resources/Images/Generator.png",
];

pub struct EntityBuilder {
	id: u16,
	position: (u16, u16),
	locations: (Vec<(Tile, (u16, u16))>, bool),
}

pub enum EntityBuilderSignals<'a> {
	Complete(io::Result<BoxCode<'a>>),
    GetTile(&'static str),
	InvalidId,
}

impl EntityBuilder {
	pub fn new(id: u16, position: (u16, u16)) -> EntityBuilder {
		EntityBuilder {
			id,
			position,
		    locations: (vec![], false),
        }
	}
	pub fn build<'a>(&self, creator: &'a TextureCreator<WindowContext>) -> EntityBuilderSignals<'a> {
		match self.id {
			0 => EntityBuilderSignals::Complete(Player::new(creator, self.position.0 as f32 * 50f32, self.position.1 as f32 * 50f32)),
			1 => EntityBuilderSignals::Complete(Skeleton::new(creator, (self.position.0 as f32 * 50f32, self.position.1 as f32 * 50f32))),
			2 => {
			    if self.locations.1 {
                    EntityBuilderSignals::Complete(Generator::new(creator, (self.position.0 as i32 * 50, self.position.1 as i32 * 50), self.locations.0.clone()))
                }
                else {
                    EntityBuilderSignals::GetTile("Pick the next tile")
                }
			},
			MAX_ENTITY_IDX.. => EntityBuilderSignals::InvalidId,
		}
	}
    pub fn addTile(&mut self, tile: Tile, location: (u16, u16)) {
        if location == self.position {self.locations.1 = true;}
        else {self.locations.0.push((tile, location));}
    }
	pub fn addEntityGlobal<'a>(&self, ctx: &mut GameContext<'a>, entity: BoxCode<'a>) {
		match self.id {
			0 => ctx.addEntityGlobal::<Player>(entity),
			1 => ctx.addEntityGlobal::<Skeleton>(entity),
			2 => ctx.addEntityGlobal::<Generator>(entity),
            MAX_ENTITY_IDX.. => unreachable!(),
		};
	}
	pub fn addEntityActiveScreen<'a>(&self, ctx: &mut GameContext<'a>, entity: BoxCode<'a>) {
		match self.id {
			0 => ctx.addEntityActiveScreen::<Player>(entity),
			1 => ctx.addEntityActiveScreen::<Skeleton>(entity),
            2 => ctx.addEntityActiveScreen::<Generator>(entity),
			MAX_ENTITY_IDX.. => unreachable!(),
		};
	}
	pub fn getEntityRect(&self) -> Rect {
		let (w, h) = match self.id {
			0 => (50, 50),
			1 => (50, 100),
            2 => (50, 50),
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

pub const MAX_ENTITY_IDX: u16 = 2;

