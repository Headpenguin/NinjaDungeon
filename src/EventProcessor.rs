use std::cell::UnsafeCell;

use crate::{ID, CollisionType, Vector};
use crate::Entities::Traits::{EntityTraits, Entity};
use crate::Entities::Holder;
use crate::GameContext;
use crate::Scheduling::Scheduler;
use crate::MapMod::{self, Tile};

use sdl2::rect::Rect;

pub struct Key {
	lock: (),
}

impl Key {
	pub unsafe fn new() -> Key {
		Key{lock: ()}
	}
}

enum Commands {
	PlaceTile(Tile, (u16, u16)),
	PlaceTiles(Tile, (u16, u16), (u16, u16)),
    ActivateEntity(ID, bool),
	InformPlayerSnakeBoss(ID),
	InformPlayerSnakeBossDeath,
	Win,
}

pub struct PO<'a> {
	ctx: GameContext<'a>,
	purgeList: UnsafeCell<Vec<ID>>,
	commands: UnsafeCell<Vec<Commands>>,
}

struct Subscriber;

pub enum CollisionMsg {
	Damage(i32),
	Ground(Rect, Vector),
}

pub struct CounterMsg(pub i32);

pub struct Envelope<T> {
	priority: i32,
	letter: T,
	recv: ID,
	sender: ID,
}

impl<T> Envelope<T> {
	pub fn new(letter: T, recv: ID, sender: ID) -> Envelope<T> {
		Envelope {
			priority: 0,
			letter,
			recv,
			sender,
		}
	}
	pub fn getMsg(&self) -> &T {return &self.letter;}
	pub fn getSender(&self) -> ID {return self.sender;}
	pub fn getReciever(&self) -> ID {return self.recv;}
}
impl Envelope<CollisionMsg> {
	pub fn send(self, recv: &mut dyn EntityTraits, po: &PO) {recv.collide(self, po);}
}

impl Envelope<CounterMsg> {
	pub fn send(self, recv: &mut dyn EntityTraits, po: &PO) {recv.inc(self, po);}
}

impl<'a> PO<'a> {
	pub fn new(ctx: GameContext) -> PO {
		PO{
			ctx,
			purgeList: UnsafeCell::new(vec![]),
			commands: UnsafeCell::new(vec![]),
		}
	}
	pub unsafe fn getCtxMut<'b>(&'b mut self) -> &'b mut GameContext<'a> {
		&mut self.ctx
	}
	pub fn getCtx<'b>(&'b self) -> &'b GameContext<'a> {
		&self.ctx
	}
	//pub fn 
	pub fn sendCollisionMsg(&self, msg: Envelope<CollisionMsg>) -> bool {
		unsafe {
			if let Some(recv) = self.getCtx().getHolder().getMut(msg.recv.mask()) {
				msg.send(&mut *recv, self);
				true
			}
			else {false}
		}
	}
	pub fn sendCounterMsg(&self, msg: Envelope<CounterMsg>) -> bool {
		unsafe {
			if let Some(recv) = self.getCtx().getHolder().getMut(msg.recv.mask()) {
				msg.send(&mut *recv, self);
				true
			}
			else {false}
		}
	}
	pub fn win(&self) {
		unsafe {&mut *self.commands.get()}.push(Commands::Win);
	}
	pub fn spawnTile(&self, tile: Tile, location: (u16, u16)) {
		unsafe {&mut *self.commands.get()}.push(Commands::PlaceTile(tile, location));
	}
	pub fn spawnTiles(&self, tile: Tile, locationBegin: (u16, u16), locationEnd: (u16, u16)) {
		unsafe {&mut *self.commands.get()}.push(Commands::PlaceTiles(tile, locationBegin, locationEnd));
	}
    pub fn activateEntity(&self, entity: ID, global: bool) {
        unsafe {&mut *self.commands.get()}.push(Commands::ActivateEntity(entity, global));
    }
	pub fn informPlayerSnakeBoss(&self, id: ID) {
		unsafe {&mut *self.commands.get()}.push(Commands::InformPlayerSnakeBoss(id));
	}
	pub fn informPlayerSnakeBossDeath(&self) {
		unsafe {&mut *self.commands.get()}.push(Commands::InformPlayerSnakeBossDeath);
	}
	pub fn updatePosition(&mut self, id: ID, hitbox: Rect, prevHitbox: Rect) {
		self.ctx.updatePosition(id, hitbox, prevHitbox);
	}
	pub fn removeCollision(&mut self, id: ID, hitbox: Rect) {
		self.ctx.removeCollision(id, hitbox);
	}
	pub fn addToPurgeList(&self, id: ID) {
		unsafe {&mut *self.purgeList.get()}.push(id);
	}
	pub unsafe fn purge(&mut self) {
		for id in self.purgeList.get_mut() {
			self.ctx.removeEntity(*id).unwrap();
		}
		self.purgeList.get_mut().clear();
	}
	pub unsafe fn doCommands(&mut self) -> bool {
		for command in self.commands.get_mut().drain(..) {
			match command {
				Commands::PlaceTile(tile, location) => self.ctx.getMapMut().changeTile(location, tile),
				Commands::PlaceTiles(tile, locationBegin, locationEnd) => MapMod::spawnTiles(tile, locationBegin, locationEnd, self.ctx.getMapMut()),
                Commands::ActivateEntity(entity, global) => {
                    if global {
                        self.ctx.activateEntityGlobal(entity);
                    }
                    else {
                        self.ctx.activateEntityActiveScreen(entity);
                    }
                }
				Commands::InformPlayerSnakeBoss(id) => {
					self.ctx.getPlayerMut().informSnakeBoss(id);
				}
				Commands::InformPlayerSnakeBossDeath => {
					self.ctx.getPlayerMut().informSnakeBossDeath();
				}
				Commands::Win => return false,
			}
		}
		true
	}
	pub fn getEntity<'b>(&'b self, id: ID, key: Key) -> (Option<&'b (dyn EntityTraits + 'a)>, Key) {
		unsafe { (self.ctx.getHolder().get(id), key) }
	}
}

pub struct SubscriberList {
	subs: Vec<ID>,

}

