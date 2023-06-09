use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{WindowContext, Window};
use sdl2::rect::Rect;

use super::{Skeleton, Generator, EntityGenerator};
use crate::{Player, Tile};
use super::BoxCode;
use super::Traits::IDRegistration;
use crate::SpriteLoader::Sprites;
use crate::{GameContext, ID};
use std::io;

const ENTITY_SPRITES: &'static [&'static str] = &[
	"Resources/Images/Ninja_float_0__half.png",
	"Resources/Images/Skeleton_top__half.png",
    "Resources/Images/Generator.png",
    "Resources/Images/Generator2.png",
];

pub struct EntityBuilder {
	id: u16,
	position: (u16, u16),
	locations: (Vec<(Tile, (u16, u16))>, bool),
	linkedIDs: (Vec<ID>, bool),
    inactiveEntities: (Vec<(ID, bool)>, bool),
}

pub enum EntityBuilderSignals<'a> {
	Complete(io::Result<BoxCode<'a>>),
    GetTile(&'static str),
	GetEntity(&'static str),
    MakeEntityInactive(&'static str),
	InvalidId,
}

impl EntityBuilder {
	pub fn new(id: u16, position: (u16, u16)) -> EntityBuilder {
		EntityBuilder {
			id,
			position,
		    locations: (vec![], false),
			linkedIDs: (vec![], false),
            inactiveEntities: (vec![], false),
        }
	}
	pub fn build<'a>(&self, creator: &'a TextureCreator<WindowContext>) -> EntityBuilderSignals<'a> {
		match self.id {
			0 => EntityBuilderSignals::Complete(Player::new(creator, self.position.0 as f32 * 50f32, self.position.1 as f32 * 50f32)),
			1 => EntityBuilderSignals::Complete(Skeleton::new(creator, (self.position.0 as f32 * 50f32, self.position.1 as f32 * 50f32))),
			2 => {
			    if self.locations.1 && self.linkedIDs.1 {
                    EntityBuilderSignals::Complete(Generator::new(creator, (self.position.0 as i32 * 50, self.position.1 as i32 * 50), self.locations.0.clone(), self.linkedIDs.0.len() as u8))
                }
				else if self.locations.1 {
					EntityBuilderSignals::GetEntity("Pick entities to link")
				}
                else {
                    EntityBuilderSignals::GetTile("Pick the next tile")
                }
			},
            3 => {
                if self.locations.1 && self.linkedIDs.1 && self.inactiveEntities.1 {
                    EntityBuilderSignals::Complete(EntityGenerator::new(creator, (self.position.0 as i32 * 50, self.position.1 as i32 * 50), self.locations.0.clone(), self.inactiveEntities.0.clone(), self.linkedIDs.0.len() as u8))
                    
                }
                else if self.locations.1 && self.linkedIDs.1 {
                    EntityBuilderSignals::MakeEntityInactive("Place the entity to spawn")
                }
				else if self.locations.1 {
					EntityBuilderSignals::GetEntity("Pick entities to link")
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
	pub fn addLinkedID(&mut self, id: ID) {
		self.linkedIDs.0.push(id);
	}
    pub fn addInactiveEntity(&mut self, id: ID, global: bool) {
        self.inactiveEntities.0.push((id, global));
    }
	pub fn endList(&mut self) {
		match self.id {
			0..=1 => (),
			2 => {
				if !self.locations.1 {self.locations.1 = true;}
				else if !self.linkedIDs.1 {self.linkedIDs.1 = true;}
			},
            3 => {
                if !self.locations.1 {self.locations.1 = true;}
                else if !self.linkedIDs.1 {self.linkedIDs.1 = true;}
                else if !self.inactiveEntities.1 {self.inactiveEntities.1 = true;}
            }
			MAX_ENTITY_IDX.. => unreachable!(),
		};
	}
	pub fn addEntityGlobal<'a>(&self, ctx: &mut GameContext<'a>, entity: BoxCode<'a>) {
		match self.id {
			0 => ctx.addEntityGlobal::<Player>(entity),
			1 => ctx.addEntityGlobal::<Skeleton>(entity),
			2 => {
				let genID = ctx.addEntityGlobal::<Generator>(entity);
				if let Some(genID) = genID {
					for id in self.linkedIDs.0.iter() {
						ctx.getHolderMut().getMutSafe(*id).unwrap().register(IDRegistration::DeathCounter(genID));
					}
				}
				genID
			},
            3 => {
                let genID = ctx.addEntityGlobal::<EntityGenerator>(entity);
                if let Some(genID) = genID {
					for id in self.linkedIDs.0.iter() {
						ctx.getHolderMut().getMutSafe(*id).unwrap().register(IDRegistration::DeathCounter(genID));
					}
                }
                genID
            },
            MAX_ENTITY_IDX.. => unreachable!(),
		};
	}
	pub fn addEntityActiveScreen<'a>(&self, ctx: &mut GameContext<'a>, entity: BoxCode<'a>) {
		match self.id {
			0 => ctx.addEntityActiveScreen::<Player>(entity),
			1 => ctx.addEntityActiveScreen::<Skeleton>(entity),
            2 => {
				let genID = ctx.addEntityActiveScreen::<Generator>(entity);
				if let Some(genID) = genID {
					for id in self.linkedIDs.0.iter() {
						ctx.getHolderMut().getMutSafe(*id).unwrap().register(IDRegistration::DeathCounter(genID));
					}
				}
				genID
			},
            3 => {
                let genID = ctx.addEntityActiveScreen::<EntityGenerator>(entity);
                if let Some(genID) = genID {
					for id in self.linkedIDs.0.iter() {
						ctx.getHolderMut().getMutSafe(*id).unwrap().register(IDRegistration::DeathCounter(genID));
					}
                }
                genID
            },
			MAX_ENTITY_IDX.. => unreachable!(),
		};
	}
	pub fn addEntityInactive<'a>(&self, ctx: &mut GameContext<'a>, entity: BoxCode<'a>) -> Option<ID> {
		if unsafe {match self.id {
			0 => ctx.getHolderMut().add::<Player>(entity),
			1 => ctx.getHolderMut().add::<Skeleton>(entity),
            2 => {
				let success = ctx.getHolderMut().add::<Generator>(entity);
				if success {
					let genID = ctx.getHolder().getCurrentID();
					for id in self.linkedIDs.0.iter() {
						ctx.getHolderMut().getMutSafe(*id).unwrap().register(IDRegistration::DeathCounter(genID));
					}
				}
				success
			},
            3 => {
                let success = ctx.getHolderMut().add::<EntityGenerator>(entity);
                if success {
					let genID = ctx.getHolder().getCurrentID();
					for id in self.linkedIDs.0.iter() {
						ctx.getHolderMut().getMutSafe(*id).unwrap().register(IDRegistration::DeathCounter(genID));
					}
                }
				success
            },
			MAX_ENTITY_IDX.. => unreachable!(),
		}} {Some(ctx.getHolder().getCurrentID())} else {None}
	}

	pub fn getEntityRect(&self) -> Rect {
		let (w, h) = match self.id {
			0 => (50, 50),
			1 => (50, 100),
            2|3 => (50, 50),
			MAX_ENTITY_IDX.. => unreachable!(),
		};
		Rect::new(self.position.0 as i32 * 50, self.position.1 as i32 * 50, w, h)
	}
    pub unsafe fn destroy(entity: BoxCode, ctx: &mut GameContext) -> Result<(), &'static str> {
        match entity {
            BoxCode::EntityGenerator(mut e) => {
                e.destroy(ctx)
            },
            _ => Ok(()),
        }
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

pub const MAX_ENTITY_IDX: u16 = 3;

