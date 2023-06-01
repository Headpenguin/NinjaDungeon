use sdl2::video::WindowContext;
use sdl2::render::TextureCreator;
use sdl2::rect::Rect;

use crate::{ID, Map, Player, Vec2d};
use crate::Entities::{Holder, TypedID, Skeleton};
use crate::Entities::Traits::EntityDyn;

pub struct GameContext<'a> {
	pub holder: Holder<'a>,
	pub map: Map<'a>,
	pub player: TypedID<'a, Player<'a>>,
	collision: Vec2d<EntityHitbox>,
	collisionCandidates: Vec<(EntityHitbox, EntityHitbox)>,
}

#[derive(Copy, Clone)]
struct EntityHitbox {
	id: ID,
	hitbox: Rect,
}

impl EntityHitbox {
	fn empty() -> EntityHitbox {
		EntityHitbox {
			id: ID::empty(),
			hitbox: Rect::new(0, 0, 0, 0),
		}
	}
	fn isEmpty(&self) -> bool {
		self.id.isEmpty()
	}
}

impl<'a> GameContext<'a> {
	pub fn new(map: Map<'a>, creator: &'a TextureCreator<WindowContext>) -> GameContext<'a> {
		let mut holder = Holder::new();
		unsafe {holder.add::<Player>(Player::new(creator, 50f32, 50f32).unwrap())};
		let player = TypedID::new(holder.getCurrentID());
		unsafe { holder.add::<Skeleton>(Skeleton::new(creator, (50f32, 50f32)).unwrap())};
		GameContext {
			holder,
			map,
			player, 
			collision: Vec2d::new(vec![EntityHitbox::empty(); 17*12], 17),
			collisionCandidates: vec![],
		}
	}
	
	pub fn updatePosition<'b>(&'b mut self, id: ID, hitbox: Rect, prevHitbox: Rect) {
		self.removeCollision(id, prevHitbox);

		let iter = self.map.calculateCollisionBounds(hitbox);
		let entry = EntityHitbox {id, hitbox};
		for location in iter {
			let tmp = self.collision.indexMut(location.1 as usize, location.0 as usize);
			if !tmp.isEmpty() && Self::getCollisionListInternal(&self.collisionCandidates, id).find(|e| e.1.id == tmp.id).is_none() {
				self.collisionCandidates.push((entry, *tmp));
				let length = Self::getCollisionListInternal(&self.collisionCandidates, tmp.id).enumerate().last().map(|e| e.0 + 1).unwrap_or(0) + self.collisionCandidates.len();
				let prevLength = self.collisionCandidates.len();
				self.collisionCandidates.resize(length, (EntityHitbox::empty(), EntityHitbox::empty()));
				{
					let (candidates, empty) = self.collisionCandidates.split_at_mut(prevLength);
					let mut iter = Self::getCollisionListInternal(candidates, tmp.id).map(|e| (entry, e.1));
					empty.fill_with(|| iter.next().unwrap());
				}
				//self.collisionCandidates.extend(Self::getCollisionListInternal(&self.collisionCandidates, tmp.id));

				self.collisionCandidates.push((*tmp, entry));
			}
			*tmp = entry;
		}
	}
	pub fn removeCollision(&mut self, id: ID, hitbox: Rect) {
		let iter = self.map.calculateCollisionBounds(hitbox);
		for location in iter {
			let tmp = self.collision.indexMut(location.1 as usize, location.0 as usize);
			if tmp.id == id {*tmp = EntityHitbox::empty();}
		}
	}
	fn getCollisionListInternal<'b>(candidates: &'b [(EntityHitbox, EntityHitbox)], id: ID) -> impl Iterator<Item=(EntityHitbox, EntityHitbox)> + 'b {
		candidates.iter().filter(move |e| e.0.id == id).map(|e| *e)
	}
	pub fn getCollisionList<'b>(&'b self, id: ID) -> impl Iterator<Item=ID> + 'b {
		Self::getCollisionListInternal(&self.collisionCandidates, id).filter(|e| e.0.hitbox.has_intersection(e.1.hitbox)).map(|e| e.1.id)
	}
	pub fn resetCollisionLists<'b>(&'b mut self) {
		self.collisionCandidates.clear();
	}

	pub fn getMap(&self) -> &Map {
		&self.map
	}
	pub fn getMapMut<'b>(&'b mut self) -> &'b mut Map<'a> {
		&mut self.map
	}
	pub fn getPlayerMut<'b>(&'b mut self) -> &'b mut Player<'a> {
		self.holder.getMutTyped(self.player).unwrap()
	}
	pub fn getPlayerID<'b>(&'b self) -> TypedID<'a, Player<'a>> {
		self.player
	}
	pub fn getHolderMut<'b>(&'b mut self) -> &'b mut Holder<'a> {
		&mut self.holder
	}
	pub fn getHolder<'b>(&'b self) -> &'b Holder<'a> {
		&self.holder
	}
	pub unsafe fn entityIter<'b>(&'b self) -> impl Iterator<Item = (ID, &'b (dyn EntityDyn + 'a))> {
		self.holder.iter()
	}
	pub unsafe fn entityIterMut<'b>(&'b mut self) -> impl Iterator<Item = (ID, &'b mut (dyn EntityDyn + 'a))> {
		self.holder.iterMut()
	}
}
