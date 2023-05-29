use sdl2::video::Window;
use sdl2::render::Canvas;

use crate::EventProcessor::{Envelope, CollisionMsg, PO};
use super::{RefCode, RefCodeMut};
use crate::GameContext;

use std::ops::{Deref, DerefMut};

pub trait Collision {
	fn collide(&mut self, _msg: Envelope<CollisionMsg>) {}
}

pub trait EntityTraits : Collision {}
impl<T> EntityTraits for T where
	T: Collision, {}

pub trait EntityTraitsWrappable<'a> : EntityTraits {
	type Data;
	fn mapCodeMut(code: RefCodeMut<'a>) -> Option<&'a mut Self>;
	fn mapCode(code: RefCode<'a>) -> Option<&'a Self>;
	fn getData(&self, data: &mut Self::Data, ctx: &GameContext);
	fn update(&mut self, data: &Self::Data, po: &PO);
	fn needsExecution(&self) -> bool;
	fn tick(&mut self);
	fn draw(&self, canvas: &mut Canvas<Window>);
}
pub struct Entity<'a, T: EntityTraitsWrappable<'a>> {
	entity: T,
	data: T::Data,
}

impl<'a, T: EntityTraitsWrappable<'a>> Deref for Entity<'a, T> {
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.entity
	}
}

impl<'a, T: EntityTraitsWrappable<'a>> DerefMut for Entity<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.entity
	}
}

impl<'a, T: EntityTraitsWrappable<'a>> Entity<'a, T> {
	pub fn new(entity: T, data: T::Data) -> Entity<'a, T> {
		Entity {
			entity,
			data,
		}
	}
}

pub trait EntityDyn {
	fn getData(&mut self, ctx: &GameContext);
	fn update(&mut self, po: &PO);
	fn getInner(&self) -> &dyn EntityTraits;
	fn getInnerMut(&mut self) -> &mut dyn EntityTraits;
	fn needsExecution(&self) -> bool;
	fn tick(&mut self);
	fn draw(&self, canvas: &mut Canvas<Window>);
}

impl<'a, T: EntityTraitsWrappable<'a>> EntityDyn for Entity<'a, T> {
	fn getData(&mut self, ctx: &GameContext) {
		self.entity.getData(&mut self.data, ctx);
	}
	fn update(&mut self, po: &PO) {
		self.entity.update(&self.data, po);
	}
	fn getInnerMut(&mut self) -> &mut dyn EntityTraits {
		&mut self.entity
	}
	fn getInner(&self) -> &dyn EntityTraits {
		&self.entity
	}
	fn needsExecution(&self) -> bool {
		self.entity.needsExecution()
	}
	fn tick(&mut self) {
		self.entity.tick();
	}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		self.entity.draw(canvas);
	}
}

