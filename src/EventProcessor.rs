use crate::{ID, CollisionType};
use crate::Entities::Traits::Entity;
use crate::Entities::Holder;

struct PO {}

struct subscriber;

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

impl<CollisionMsg> Envelope<CollisionMsg> {
	pub fn send(self, recv: &mut dyn EntityTraits) {recv.collide(self);}
}

impl PO {
	fn update(&mut self, holder: &mut Holder, scheduler: &Scheduling, ctx: &GameContext) {
		scheduler.needsExecution().forEach(|id| unsafe { (holder.getEntityDyn() as &mut dyn EntityDyn).getData(ctx) });
		scheduler.needsExecution().forEach(|id| unsafe { (holder.getEntityDyn() as &mut dyn EntityDyn).update() });
	}
	fn sendMsg<T>(&self, holder: &mut Holder, msg: Envelope<T>) {
		unsafe {
			msg.send(holder.getMut(msg.recv) as &mut dyn Entity);
		}
	}
}

pub struct SubscriberList {
	subs: Vec<ID>,

}

