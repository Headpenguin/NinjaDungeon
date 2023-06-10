use sdl2::video::WindowContext;
use sdl2::render::TextureCreator;
use sdl2::rect::Rect;

use serde::{Serialize, Deserialize};

use crate::{ID, Map, Player, Vec2d};
use crate::Entities::{Holder, InnerHolder, TypedID, Skeleton, BoxCode};
use crate::Entities::Traits::{EntityDyn, EntityTraitsWrappable};
use crate::IntHasher::UInt64Hasher;
use crate::MapMod::InnerMap;

use std::collections::HashSet;
use std::io;

#[derive(Serialize, Deserialize)]
pub struct InnerGameContext {
	holder: InnerHolder,
	map: InnerMap,
	player: ID,
	globalEntities: HashSet<u64, UInt64Hasher>,
}

impl InnerGameContext {
	pub unsafe fn fromGameContext(ctx: &GameContext) -> InnerGameContext {
		InnerGameContext {
			holder: InnerHolder::fromHolder(&ctx.holder),
			map: (&ctx.map as &InnerMap).clone(),
			player: ctx.player.getID(),
			globalEntities: ctx.globalEntities.clone(),
		}
	}
	pub fn intoGameContext<'a>(self, creator: &'a TextureCreator<WindowContext>) -> io::Result<GameContext<'a>> {
		Ok(GameContext {
			holder: self.holder.intoHolder(creator)?,
			map: Map::restore(self.map, 0, "Resources/Images/Map1.anim", creator)?,
			player: TypedID::new(self.player),
			collision: Vec2d::new(vec![EntityHitbox::empty(); 17*12], 17),
			collisionCandidates: vec![],
			globalEntities: self.globalEntities,
		})
	}
}

pub struct GameContext<'a> {
	pub holder: Holder<'a>,
	pub map: Map<'a>,
	pub player: TypedID<'a, Player<'a>>,
	collision: Vec2d<EntityHitbox>,
	collisionCandidates: Vec<(EntityHitbox, EntityHitbox)>,
	globalEntities: HashSet<u64, UInt64Hasher>,
}

#[derive(Copy, Clone, Debug)]
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
		let holder = Holder::new();
		//unsafe {holder.add::<Player>(Player::new(creator, 50f32, 50f32).unwrap())};
		//unsafe { holder.add::<Skeleton>(Skeleton::new(creator, (50f32, 50f32)).unwrap())};

		let mut ctx = GameContext {
			holder,
			map,
			player: TypedID::new(ID::empty()), 
			collision: Vec2d::new(vec![EntityHitbox::empty(); 17*12], 17),
			collisionCandidates: vec![],
			globalEntities: HashSet::default(),
		};
		ctx.addEntityGlobal::<Player>(Player::new(creator, 50f32, 50f32).unwrap());
		ctx.player = TypedID::new(ctx.holder.getCurrentID());
		ctx
	}
	pub fn addEntityActiveScreen<T: EntityTraitsWrappable<'a> + 'a>(&mut self, entity: BoxCode<'a>) -> Option<ID> {
		unsafe {
			if self.holder.add::<T>(entity) {
				let id = self.holder.getCurrentID();
				self.map.addEntityActiveScreen(id);
				Some(id)
			}
			else {
				None
			}
		}
	}
	pub fn addEntityGlobal<T: EntityTraitsWrappable<'a> + 'a>(&mut self, entity: BoxCode<'a>) -> Option<ID> {
		unsafe {
			if self.holder.add::<T>(entity) {
				let id = self.holder.getCurrentID();
				self.globalEntities.insert(id.getID());
				Some(id)
			}
			else {
				None
			}
		}
	}
    pub unsafe fn activateEntityGlobal(&mut self, id: ID) {
        self.globalEntities.insert(id.getID());
    }
    pub unsafe fn activateEntityActiveScreen(&mut self, id: ID) {
        self.map.addEntityActiveScreen(id);
    }
	pub unsafe fn removeEntity(&mut self, id: ID) -> Result<BoxCode<'a>, (Option<BoxCode<'a>>, &'static str)> {
		let res = self.holder.remove(id);
		if self.globalEntities.remove(&id.getID()) {
			res.ok_or((None, "Entity does not exist and global entities are corrupted"))
		}
		else if self.map.removeEntityActiveScreen(id) {
			res.ok_or((None, "Entity does not exist and active screen entities are corrupted"))
		}
		else {
			Err((res, "Entity not found in active screen or globally"))
		}
	}
	
	pub fn updatePosition<'b>(&'b mut self, id: ID, hitbox: Rect, prevHitbox: Rect) {
		self.removeCollisionInternal(id, prevHitbox);

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
	fn removeCollisionInternal(&mut self, id: ID, hitbox: Rect) {
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
	pub fn removeCollision(&mut self, id: ID, hitbox: Rect) {
		self.removeCollisionInternal(id, hitbox);
		self.collisionCandidates.retain(|pair| pair.0.id != id && pair.1.id != id);
	}
	pub fn disableEntityCollisionFrame(&mut self) {
		self.collision.fill(EntityHitbox::empty());
	}
	pub fn resetCollisionLists<'b>(&'b mut self) {
		self.collisionCandidates.clear();
	}

	pub unsafe fn getEntityAtPositionActiveScreen(&self, hitbox: Rect) -> Option<ID> {
		for id in self.activeScreenEntityIter() {
			if self.holder.getRefCode(id).unwrap().collidesStatic(hitbox) {
				return Some(id);
			}
		}
		None
	}
	pub unsafe fn getEntityAtPositionGlobal(&self, hitbox: Rect) -> Option<ID> {
		for id in self.globalEntityIter() {
			if self.holder.getRefCode(id).unwrap().collidesStatic(hitbox) {
				return Some(id);
			}
		}
		None
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
	pub unsafe fn entityIter<'b>(&'b self) -> impl Iterator<Item = (ID, &'b (dyn EntityDyn<'a> + 'a))> {
		self.holder.iter()
	}
	pub unsafe fn entityIterMut<'b>(&'b mut self) -> impl Iterator<Item = (ID, &'b mut (dyn EntityDyn<'a> + 'a))> {
		self.holder.iterMut()
	}
	pub unsafe fn globalEntityIter<'b>(&'b self) -> impl Iterator<Item = ID> + 'b {
		self.globalEntities.iter().map(|id| ID::new(*id, 0))
	}
	pub unsafe fn activeScreenEntityIter<'b>(&'b self) -> impl Iterator<Item = ID> + 'b {
		self.map.getScreen(self.map.getActiveScreenId()).unwrap().getEntitiesIter()
	}
}
