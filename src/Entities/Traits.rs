use crate::EventProcessor::{Envelope, CollisionMsg,};
use crate::GameContext;

pub trait Collision {
	fn collide(&mut self, _msg: Envelope<CollisionMsg>) {}
}
pub trait EntityTraits : Collision {}
pub trait EntityTraitsWrappable<D> : EntityTraits {
	fn getData(&self, data: &mut D, ctx: &GameContext);
	fn update(&mut self, data: &D);
}
pub struct Entity<D, T: EntityTraitsWrappable<D>> {
	entity: T,
	data: D,
}

pub trait EntityDyn {
	fn getData(&mut self, ctx: &GameContext);
	fn update(&mut self);
	fn getInner(&self) -> &dyn EntityTraits;
	fn getInnerMut(&mut self) -> &mut dyn EntityTraits;
}

impl<D, T: EntityTraitsWrappable<D>> EntityDyn for Entity<D, T> {
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

