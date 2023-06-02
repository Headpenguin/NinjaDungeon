use std::cell::UnsafeCell;

use crate::{ID, CollisionType};
use crate::Entities::Traits::EntityTraits;
use crate::Entities::Holder;
use crate::GameContext;
use crate::Scheduling::Scheduler;

use sdl2::rect::Rect;

pub struct PO<'a> {
	ctx: GameContext<'a>,
	purgeList: UnsafeCell<Vec<ID>>,
}

struct Subscriber;

pub enum CollisionMsg {
	Damage(i32),
}

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
}
impl Envelope<CollisionMsg> {
	pub fn send(self, recv: &mut dyn EntityTraits, po: &PO) {recv.collide(self, po);}
}

impl<'a> PO<'a> {
	pub fn new(ctx: GameContext) -> PO {
		PO{
			ctx,
			purgeList: UnsafeCell::new(vec![]),
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
			self.ctx.getHolderMut().remove(*id);
		}
		self.purgeList.get_mut().clear();
	}
}

pub struct SubscriberList {
	subs: Vec<ID>,

}

