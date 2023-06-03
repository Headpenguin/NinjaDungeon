use sdl2::video::Window;
use sdl2::render::Canvas;

use crate::EventProcessor::{Envelope, CollisionMsg, PO, Key};
use super::{RefCode, RefCodeMut, TypedID};
use crate::{GameContext, ID};

use std::ops::{Deref, DerefMut};

pub trait Collision {
	fn collide(&mut self, _msg: Envelope<CollisionMsg>, _po: &PO) {}
	fn collideWith(&self, other: ID, po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key);
}

pub trait EntityTraits : Collision {}
impl<T> EntityTraits for T where
	T: Collision, {}

pub trait EntityTraitsWrappable<'a> : EntityTraits where Self: Sized {
	type Data;
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self>;
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self>;
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key;
	fn update(&mut self, data: &Self::Data, po: &mut PO);
	fn needsExecution(&self) -> bool;
	fn tick(&mut self);
	fn draw(&self, canvas: &mut Canvas<Window>);
	fn setID(&mut self, id: TypedID<'a, Self>);
}

#[derive(Debug)]
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
	fn getData(&mut self, ctx: &PO, key: Key) -> Key;
	fn update(&mut self, po: &mut PO);
	fn getInner(&self) -> &dyn EntityTraits;
	fn getInnerMut(&mut self) -> &mut dyn EntityTraits;
	fn needsExecution(&self) -> bool;
	fn tick(&mut self);
	fn draw(&self, canvas: &mut Canvas<Window>);
}

impl<'a, T: EntityTraitsWrappable<'a>> EntityDyn for Entity<'a, T> {
	fn getData(&mut self, po: &PO, key: Key) -> Key {
		self.entity.getData(&mut self.data, po, key)
	}
	fn update(&mut self, po: &mut PO) {
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

