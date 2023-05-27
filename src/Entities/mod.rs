pub mod Traits;

use Traits::{Entity, EntityDyn, EntityTraits, EntityTraitsWrappable};
use std::collections::HashMap;
use std::cell::UnsafeCell;
use crate::ID;
use crate::IntHasher::IntHasher;
use crate::Player;
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

pub struct TypedID<'a, T: EntityTraitsWrappable<'a>>(ID, PhantomData<&'a T>);

impl<'a, T: EntityTraitsWrappable<'a>> TypedID<'a, T> {
	pub fn new(id: ID) -> TypedID<'a, T> {
		TypedID(id, PhantomData)
	}
}

pub enum BoxCode<'a> {
	Player(Box<Entity<'a, Player<'a>>>),
}

pub enum RefCode<'a> {
	Player(&'a mut Entity<'a, Player<'a>>),
}

impl<'a> BoxCode<'a> {
	pub fn refMut(&'a mut self) -> RefCode<'a> {
		match self {
			BoxCode::Player(ref mut p) => RefCode::Player(p),
		}
	}
}

impl<'a> Deref for BoxCode<'a> {
	type Target = dyn EntityDyn + 'a;
	fn deref(&self) -> &Self::Target {
		match self {
			Self::Player(p) => p as &Entity<Player> as &(dyn EntityDyn + 'a),
		}
	}
}
impl<'a> DerefMut for BoxCode<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Player(p) => p as &mut Entity<Player> as &mut (dyn EntityDyn + 'a),
		}
	}
}

pub struct Holder<'a> {
	entities: HashMap<ID, UnsafeCell<BoxCode<'a>>, IntHasher>,
	currentId: ID,
}

impl<'a> Holder<'a> {
	pub fn getMutTyped<T: EntityTraitsWrappable<'a>>(&mut self, id: TypedID<'a, T>) -> Option<&mut T> {
		unsafe {self.entities.get_mut(&id.0).map(|x| <T>::mapCode((&mut *x.get()).refMut())).flatten()}
	}
	pub unsafe fn get(&self, id: ID) -> Option<&dyn EntityTraits> {
		self.entities.get(&id).map(|x| unsafe{
			(&*x.get()).getInner() as &dyn EntityTraits
		})
	}
	pub unsafe fn getMut(&self, id: ID) -> Option<*mut (dyn EntityTraits + 'a)> {
		self.entities.get(&id).map(|x| unsafe {
			(&mut *x.get()).getInnerMut() as *mut dyn EntityTraits
		})
	}
	pub unsafe fn getEntityDyn(&self, id: ID) -> Option<*mut (dyn EntityDyn + 'a)> {
		self.entities.get(&id).map(|x| unsafe {
			(&mut *x.get()).deref_mut() as *mut dyn EntityDyn
		})
	}
	pub unsafe fn add(&mut self, entity: BoxCode<'a>) {
		self.entities.insert(self.currentId, UnsafeCell::new(entity));
		self.currentId += 1;
	}
	pub unsafe fn remove(&'a mut self, id: ID) -> Option<BoxCode<'a>> {
		self.entities.remove(&id).map(|x| x.into_inner())
	}
	pub fn new() -> Holder<'a> {
		Holder {
			entities: HashMap::default(),
			currentId: 0,
		}
	}
}

