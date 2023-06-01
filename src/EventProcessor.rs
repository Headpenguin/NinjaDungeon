use std::cell::UnsafeCell;

use crate::{ID, CollisionType};
use crate::Entities::Traits::EntityTraits;
use crate::Entities::Holder;
use crate::GameContext;
use crate::Scheduling::Scheduler;

use sdl2::rect::Rect;

pub struct PO<'a> {
	ctx: GameContext<'a>,
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
	pub fn getMsg(&self) -> &T {return &self.letter;}
	pub fn getSender(&self) -> ID {return self.sender;}
}
impl Envelope<CollisionMsg> {
	pub fn send(self, recv: &mut dyn EntityTraits) {recv.collide(self);}
}

impl<'a> PO<'a> {
	pub fn new(ctx: GameContext) -> PO {
		PO{
			ctx,
		}
	}
	pub unsafe fn getCtxMut<'b>(&'b mut self) -> &'b mut GameContext<'a> {
		&mut self.ctx
	}
	pub fn getCtx<'b>(&'b self) -> &'b GameContext<'a> {
		&self.ctx
	}
	//pub fn 
	pub unsafe fn update(&mut self, scheduler: &Scheduler) {
	}
	pub fn sendCollisionMsg(&self, holder: &mut Holder, msg: Envelope<CollisionMsg>) -> bool {
		unsafe {
			if let Some(recv) = holder.getMut(msg.recv.mask()) {
				msg.send(&mut *recv);
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
}

pub struct SubscriberList {
	subs: Vec<ID>,

}

