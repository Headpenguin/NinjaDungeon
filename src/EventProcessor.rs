use crate::{ID, CollisionType};
use crate::Entities::Traits::EntityTraits;
use crate::Entities::Holder;
use crate::GameContext;
use crate::Scheduling::Scheduler;

struct PO {}

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

impl PO {
	fn update(&mut self, holder: &mut Holder, scheduler: &Scheduler, ctx: &GameContext) {
		scheduler.needsExecution().for_each(|id| unsafe { (&mut *holder.getEntityDyn(id).unwrap()).getData(ctx) });
		scheduler.needsExecution().for_each(|id| unsafe { (&mut *holder.getEntityDyn(id).unwrap()).update() });
	}
	fn sendCollisionMsg(&self, holder: &mut Holder, msg: Envelope<CollisionMsg>) -> bool {
		unsafe {
			if let Some(recv) = holder.getMut(msg.recv) {
				msg.send(&mut *recv);
				true
			}
			else {false}
		}
	}
}

pub struct SubscriberList {
	subs: Vec<ID>,

}

