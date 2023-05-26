pub mod Traits;

use Traits::{Entity, EntityDyn, EntityTraits, EntityTraitsWrappable};
use std::collections::HashMap;
use std::cell::UnsafeCell;
use crate::ID;
use crate::IntHasher::IntHasher;

pub enum Codes {
	Skeleton,
}

pub struct Holder {
	entities: HashMap<ID, UnsafeCell<Box<dyn EntityDyn>>, IntHasher>,
	currentId: ID,
}

impl Holder {
	pub unsafe fn get(&self, id: ID) -> Option<&dyn EntityTraits> {
		self.entities.get(&id).map(|x| unsafe{
			(&*x.get()).getInner() as &dyn EntityTraits
		})
	}
	pub unsafe fn getMut(&self, id: ID) -> Option<*mut dyn EntityTraits> {
		self.entities.get(&id).map(|x| unsafe {
			(&mut *x.get()).as_mut().getInnerMut() as *mut dyn EntityTraits
		})
	}
	pub unsafe fn getEntityDyn(&self, id: ID) -> Option<*mut dyn EntityDyn> {
		self.entities.get(&id).map(|x| unsafe {
			(&mut *x.get()).as_mut() as *mut dyn EntityDyn
		})
	}
	pub unsafe fn add(&mut self, entity: Box<dyn EntityDyn>) {
		self.entities.insert(self.currentId, UnsafeCell::new(entity));
		self.currentId += 1;
	}
	pub unsafe fn remove(&mut self, id: ID) -> Option<Box<dyn EntityDyn>> {
		self.entities.remove(&id).map(|x| x.into_inner())
	}
	pub fn new() -> Holder {
		Holder {
			entities: HashMap::default(),
			currentId: 0,
		}
	}
}

