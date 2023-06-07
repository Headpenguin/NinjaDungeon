use crate::ID;
use crate::EventProcessor::{PO, Envelope, CounterMsg};

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub struct DeathCounter {
    dst: ID,
}

impl DeathCounter {
	pub fn new(dst: ID) -> Self {
		DeathCounter {
			dst,
		}
	}
	pub fn inc(&self, sender: Option<ID>, amt: i32, po: &PO) {
		po.sendCounterMsg(Envelope::new(CounterMsg(amt), self.dst, sender.unwrap_or(ID::empty())));
	}
}
