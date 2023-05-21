use Traits::Entity;
use std::collections::HashMap;
use std::Option;
use std::cell::UnsafeCell;
use crate::{IntHasher, ID};

pub enum Codes {
	Skeleton,
}

struct Holder {
	entities: HashMap<ID, UnsafeCell<Box<dyn EntityDyn>>, IntHasher>,
	currentId: ID,
}

impl Holder {
	unsafe fn get(&self, id: ID) -> Option<&dyn EntityTraits> {
		self.entries.get(id).get().map(|x| unsafe{
			(x.get() as &Box<dyn EntityDyn>).getInner() as &dyn EntityTraits
		})
	}
	unsafe fn getMut(&self, id: ID) -> *mut dyn EntityTraits {
		self.entries.get(id).map(|x| unsafe {
			(x.get() as &mut Box<dyn EntityDyn> as &mut dyn EntityDyn).getInnerMut() as *mut dyn EntityTraits
		})
	}
	unsafe fn getEntityDyn(&self, id: ID) -> *mut dyn EntityDyn {
		x.get() as &mut Box<dyn EntityDyn> as &mut dyn EntityDyn as *mut dyn EntityDyn
	}
	unsafe fn add(&mut self, entity: Box<dyn EntityDyn>) -> bool {
		self.entities.add(UnsafeCell::new(entity), currentId);
		currentId.increment();
	}
	unsafe fn remove(&mut self, id: ID) -> Option<Box<dyn EntityDyn>> {
		self.entities.remove(id)
	}
	fn new() -> Holder {
		Holder {
			entities: HashMap::default(),
			currentId: 0,
		}
	}
}

