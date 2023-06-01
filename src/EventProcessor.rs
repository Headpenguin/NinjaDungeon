use std::cell::UnsafeCell;

use crate::{ID, CollisionType};
use crate::Entities::Traits::EntityTraits;
use crate::Entities::Holder;
use crate::GameContext;
use crate::Scheduling::Scheduler;

use sdl2::rect::Rect;

pub struct PO<'a> {
	ctx: UnsafeCell<GameContext<'a>>,
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
			ctx: UnsafeCell::new(ctx),
		}
	}
	pub fn getCtx<'b>(&'b mut self) -> &'b mut GameContext<'a> {
		self.ctx.get_mut()
	}
	pub unsafe fn update(&mut self, scheduler: &Scheduler) {
		Scheduler::tick(self.ctx.get_mut());
		scheduler.execute(&*self.ctx.get(), |id| (&mut *(&*self.ctx.get()).getHolder().getEntityDyn(id).unwrap()).getData(&*self.ctx.get()));
		self.ctx.get_mut().resetCollisionLists();
		scheduler.execute(&*self.ctx.get(), |id| (&mut *(&*self.ctx.get_mut()).getHolder().getEntityDyn(id).unwrap()).update(self) );
	}
	pub fn sendCollisionMsg(&self, holder: &mut Holder, msg: Envelope<CollisionMsg>) -> bool {
		unsafe {
			if let Some(recv) = holder.getMut(msg.recv) {
				msg.send(&mut *recv);
				true
			}
			else {false}
		}
	}

	pub fn updatePosition(&self, id: ID, hitbox: Rect, prevHitbox: Rect) {
		unsafe {&mut *self.ctx.get()}.updatePosition(id, hitbox, prevHitbox);
	}
	pub fn transition(&mut self) {
		unsafe { (&mut *self.ctx.get()).getPlayerMut().transition(self); }
	}
}

pub struct SubscriberList {
	subs: Vec<ID>,

}

