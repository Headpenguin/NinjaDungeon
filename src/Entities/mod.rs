use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::rect::Rect;

use serde::{Serialize, Deserialize};

pub mod Traits;
pub mod SkeletonMod;
mod Builder;

pub use Builder::*;

pub use SkeletonMod::Skeleton;

use SkeletonMod::InnerSkeleton;

use Traits::{Entity, EntityDyn, EntityTraits, EntityTraitsWrappable};
use std::collections::HashMap;
use std::cell::UnsafeCell;
use std::io;
use crate::ID;
use crate::IntHasher::UInt64Hasher;
use crate::PlayerMod::{Player, InnerPlayer};
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;
use std::marker::Copy;
use std::clone::Clone;

#[derive(Debug)]
pub struct TypedID<'a, T: EntityTraitsWrappable<'a>>(ID, PhantomData<&'a T>);

impl<'a, T: EntityTraitsWrappable<'a>> Clone for TypedID<'a, T> {
	fn clone(&self) -> Self {
		TypedID(self.0, PhantomData)
	}
}

impl<'a, T: EntityTraitsWrappable<'a>> Copy for TypedID<'a, T> {}

impl<'a, T: EntityTraitsWrappable<'a>> TypedID<'a, T> {
	pub fn new(id: ID) -> TypedID<'a, T> {
		TypedID(id, PhantomData)
	}
	pub fn getID(&self) -> ID {
		self.0
	}
}

#[derive(Debug)]
pub enum BoxCode<'a> {
	Player(Entity<'a, Player<'a>>),
	Skeleton(Entity<'a, Skeleton<'a>>),
}

pub enum RefCodeMut<'a, 'b> {
	Player(&'b mut Entity<'a, Player<'a>>),
	Skeleton(&'b mut Entity<'a, Skeleton<'a>>),
}

pub enum RefCode<'a, 'b> {
	Player(&'b Entity<'a, Player<'a>>),
	Skeleton(&'b Entity<'a, Skeleton<'a>>),
}

impl<'a, 'b> RefCode<'a, 'b> {
	pub fn collidesStatic(&self, hitbox: Rect) -> bool {
		match self {
			RefCode::Player(e) => e.collidesStatic(hitbox),
			RefCode::Skeleton(e) => e.collidesStatic(hitbox),
		}
	}
}

impl<'a> BoxCode<'a> {
	pub fn refcodeMut<'b>(&'b mut self) -> RefCodeMut<'a, 'b> {
		match self {
			BoxCode::Player(ref mut e) => RefCodeMut::Player(e),
			BoxCode::Skeleton(ref mut e) => RefCodeMut::Skeleton(e),
		}
	}
	pub fn refcode<'b>(&'b self) -> RefCode<'a, 'b> {
		match self {
			BoxCode::Player(ref e) => RefCode::Player(e),
			BoxCode::Skeleton(ref e) => RefCode::Skeleton(e),
		}
	}
}

impl<'a> Deref for BoxCode<'a> {
	type Target = dyn EntityDyn + 'a;
	fn deref(&self) -> &Self::Target {
		match self {
			Self::Player(e) => e as &Entity<Player> as &(dyn EntityDyn + 'a),
			Self::Skeleton(e) => e as &Entity<Skeleton> as &(dyn EntityDyn + 'a),
		}
	}
}
impl<'a> DerefMut for BoxCode<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			Self::Player(e) => e as &mut Entity<Player> as &mut (dyn EntityDyn + 'a),
			Self::Skeleton(e) => e as &mut Entity<Skeleton> as &mut (dyn EntityDyn + 'a),
		}
	}
}

#[derive(Serialize, Deserialize)]
pub enum InnerCode {
	Player(InnerPlayer),
	Skeleton(InnerSkeleton),
}

impl InnerCode {
	pub fn intoBoxCode<'a>(self, creator: &'a TextureCreator<WindowContext>) -> io::Result<BoxCode<'a>> {
		match self {
			InnerCode::Player(e) => Player::fromInner(e, creator),
			InnerCode::Skeleton(e) => Skeleton::fromInner(e, creator),
		}
	}
	pub fn fromBoxCode(code: &BoxCode) -> InnerCode {
		match code {
			BoxCode::Player(e) => InnerCode::Player(InnerPlayer::fromPlayer(e)),
			BoxCode::Skeleton(e) => InnerCode::Skeleton(InnerSkeleton::fromSkeleton(e)),
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct InnerHolder {
	innerEntities: HashMap<u64, InnerCode, UInt64Hasher>,
	currentId: u64,
}

impl InnerHolder {
	pub unsafe fn fromHolder(holder: &Holder) -> InnerHolder {
		let mut innerEntities = HashMap::default();
		for (key, entity) in holder.entities.iter() {
			innerEntities.insert(*key, InnerCode::fromBoxCode(&*entity.get()));
		}
		InnerHolder {
			innerEntities,
			currentId: holder.currentId,
		}
	}
	pub fn intoHolder<'a>(self, creator: &'a TextureCreator<WindowContext>) -> io::Result<Holder<'a>> {
		let mut entities = HashMap::default();
		for (key, entity) in self.innerEntities.into_iter() {
			entities.insert(key, UnsafeCell::new(entity.intoBoxCode(creator)?));
		}
		Ok(Holder {
			entities,
			currentId: self.currentId,
		})
	}
}

pub struct Holder<'a> {
	entities: HashMap<u64, UnsafeCell<BoxCode<'a>>, UInt64Hasher>,
	currentId: u64,
}

impl<'a> Holder<'a> {
	pub fn getMutTyped<T: EntityTraitsWrappable<'a>>(&mut self, id: TypedID<'a, T>) -> Option<&mut T> {
		unsafe {self.entities.get_mut(&id.getID().getID()).map(|x| <T>::mapCodeMut((&mut *x.get()).refcodeMut())).flatten()}
	}
	pub fn getTyped<T: EntityTraitsWrappable<'a>>(&self, id: TypedID<'a, T>) -> Option<&T> {
		unsafe {self.entities.get(&id.getID().getID()).map(|x| <T>::mapCode((&mut *x.get()).refcode())).flatten()}
	}
	pub unsafe fn get<'b>(&'b self, id: ID) -> Option<&'b (dyn EntityTraits + 'a)> {
		self.entities.get(&id.getID()).map(|x| unsafe{
			(&*x.get()).getInner() as &dyn EntityTraits
		})
	}
	pub unsafe fn getMut(&self, id: ID) -> Option<*mut (dyn EntityTraits + 'a)> {
		self.entities.get(&id.getID()).map(|x| unsafe {
			(&mut *x.get()).getInnerMut() as *mut dyn EntityTraits
		})
	}
	pub unsafe fn getRefCode<'b>(&'b self, id: ID) -> Option<RefCode<'a, 'b>> {
		self.entities.get(&id.getID()).map(|x| unsafe {
			(& *x.get()).refcode()
		})
	}
	pub unsafe fn getEntityDyn(&self, id: ID) -> Option<*mut (dyn EntityDyn + 'a)> {
		self.entities.get(&id.getID()).map(|x| unsafe {
			(&mut *x.get()).deref_mut() as *mut dyn EntityDyn
		})
	}
	pub unsafe fn add<T>(&mut self, mut entity: BoxCode<'a>) -> bool where T: EntityTraitsWrappable<'a> + 'a {
		if let Some(t) = T::mapCodeMut(entity.refcodeMut()) {
			t.setID(TypedID::new(ID::new(self.currentId, 0)));
		}
		else {return false;}
		self.entities.insert(self.currentId, UnsafeCell::new(entity));
		self.currentId += 1;	
		true
	}
	pub unsafe fn remove<'b>(&'b mut self, id: ID) -> Option<BoxCode<'a>> {
		self.entities.remove(&id.getID()).map(|x| x.into_inner())
	}
	pub unsafe fn iter<'b>(&'b self) -> impl Iterator<Item=(ID, &'b (dyn EntityDyn + 'a))> {
		self.entities.iter().map(|kv| (ID::new(*kv.0, 0), (& *kv.1.get()).deref()))
	}
	pub unsafe fn iterMut<'b>(&'b mut self) -> impl Iterator<Item=(ID, &'b mut (dyn EntityDyn + 'a))> {
		self.entities.iter_mut().map(|kv| (ID::new(*kv.0, 0), (&mut *kv.1.get()).deref_mut()))
	}
	pub fn getCurrentID(&self) -> ID {
		ID::new(self.currentId - 1, 0)
	}
	pub fn new() -> Holder<'a> {
		Holder {
			entities: HashMap::default(),
			currentId: 0,
		}
	}
}

