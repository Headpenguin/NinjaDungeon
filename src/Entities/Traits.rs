use sdl2::video::Window;
use sdl2::render::Canvas;

use crate::EventProcessor::{Envelope, CollisionMsg, PO, Key, CounterMsg};
use super::{RefCode, RefCodeMut, TypedID};
use crate::{GameContext, ID};

use std::ops::{Deref, DerefMut};

pub trait Counter {
    fn inc(&mut self, msg: Envelope<CounterMsg>, po: &PO) {}
}

pub enum IDRegistration {
	DeathCounter(ID),
}

pub trait RegisterID {
	fn register(&mut self, id: IDRegistration) {}
}

pub trait Collision {
	fn collide(&mut self, _msg: Envelope<CollisionMsg>, _po: &PO) {}
	fn collideWith(&self, id: ID, other: ID, po: &PO, key: Key) -> (Option<Envelope<CollisionMsg>>, Key) {(None, key)}
}

pub trait EntityTraits : Collision + Counter + RegisterID {}
impl<T> EntityTraits for T where
	T: Collision + Counter + RegisterID {}

pub trait EntityTraitsWrappable<'a> : EntityTraits where Self: Sized {
	type Data;
	fn mapCodeMut<'b>(code: RefCodeMut<'a, 'b>) -> Option<&'b mut Self>;
	fn mapCode<'b>(code: RefCode<'a, 'b>) -> Option<&'b Self>;
	fn getData(&self, data: &mut Self::Data, po: &PO, key: Key) -> Key;
	fn update(&mut self, data: &Self::Data, po: &mut PO);
	fn needsExecution(&self) -> bool;
	fn tick(&mut self);
	fn draw(&self, canvas: &mut Canvas<Window>);
    fn drawPriority(&self) -> u8 {0}
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

pub trait EntityDyn<'a> {
	fn getData(&mut self, ctx: &PO, key: Key) -> Key;
	fn update(&mut self, po: &mut PO);
	fn getInner(&self) -> &dyn EntityTraits;
	fn getInnerMut<'b>(&'b mut self) -> &'b mut (dyn EntityTraits + 'a);
	fn needsExecution(&self) -> bool;
	fn tick(&mut self);
	fn draw(&self, canvas: &mut Canvas<Window>);
    fn drawPriority(&self) -> u8;
}

impl<'a, T: EntityTraitsWrappable<'a> + 'a> EntityDyn<'a> for Entity<'a, T> {
	fn getData(&mut self, po: &PO, key: Key) -> Key {
		self.entity.getData(&mut self.data, po, key)
	}
	fn update(&mut self, po: &mut PO) {
		self.entity.update(&self.data, po);
	}
	fn getInnerMut<'b>(&'b mut self) -> &'b mut (dyn EntityTraits + 'a) {
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
    fn drawPriority(&self) -> u8 {
        self.entity.drawPriority()
    }
}

