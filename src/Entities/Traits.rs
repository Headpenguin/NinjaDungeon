use crate::EventProcessor::{Envelope, CollisionMsg,};

pub trait Collision : {
	pub fn collide(&mut self, msg: Envelope<CollisionMsg>) {}
}
pub trait EntityTraits : Collision, {}
pub trait EntityTraitsWrappable<D> : EntityTraits {
	pub fn getData(&self, data: &mut D, ctx: &GameContext);
	pub fn update(&mut self, data: &D);
}
pub struct Entity<T: EntityTraitsWrappable, D> {
	entity: T,
	data: D,
}

trait EntityDyn {
	pub fn getData(&mut self, ctx: &GameContext);
	pub fn update(&mut self);
	pub fn getInner(&mut self) -> &mut dyn EntityTraits;
}

impl<T: EntityTraitsWrappable, D> EntityDyn for Entity<T: EntityTraitsWrappable, D> {
	pub fn getData(&mut self, ctx: &GameContext) {
		self.entity.getData(self.data, ctx);
	}
	pub fn update(&mut self) {
		self.entity.update(self.data);
	}
	pub fn getInner(&mut self) -> &mut dyn EntityTraits {
		&mut self.entity
	}
}

