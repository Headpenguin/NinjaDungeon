use crate::EventProcessor::{Envelope, CollisionMsg,};
use super::RefCode;
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
	fn mapCode(code: RefCode<'a>) -> Option<&'a mut Self>;
	fn getData(&self, data: &mut Self::Data, ctx: &GameContext);
	fn update(&mut self, data: &Self::Data);
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

pub trait EntityDyn {
	fn getData(&mut self, ctx: &GameContext);
	fn update(&mut self);
	fn getInner(&self) -> &dyn EntityTraits;
	fn getInnerMut(&mut self) -> &mut dyn EntityTraits;
}

impl<'a, T: EntityTraitsWrappable<'a>> EntityDyn for Entity<'a, T> {
	fn getData(&mut self, ctx: &GameContext) {
		self.entity.getData(&mut self.data, ctx);
	}
	fn update(&mut self) {
		self.entity.update(&self.data);
	}
	fn getInnerMut(&mut self) -> &mut dyn EntityTraits {
		&mut self.entity
	}
	fn getInner(&self) -> &dyn EntityTraits {
		&self.entity
	}
}

