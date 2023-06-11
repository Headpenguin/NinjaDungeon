use crate::ID;
use crate::EventProcessor::{PO, Envelope, CounterMsg};

use serde::{Serialize, Deserialize};

use crate::Vector;
use crate::MapMod::CollisionType;
use std::cmp::Ordering;

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

pub fn checkLineOfSight(pos: Vector, line: Vector, po: &PO) -> bool {
	let m = line.1 / line.0;
	let (startx, endx) = if line.0 > 0f32 {
		(pos.0, line.0 + pos.0)
	} else {
		(line.0 + pos.0, pos.0)
	};
	let (starty, endy) = if line.0 > 0f32 {
		(pos.1, pos.1 + line.1)
	} else {
		(pos.1 + line.1, pos.1)
	};
	for x in ((startx as i32 / 50) as u16)..=((endx as i32 / 50) as u16) {
		let (y0, y1) = {
			let x = x as f32 * 50f32;
			let x1 = x as f32 + 50f32;
			let y = (x - startx) * m + starty;
			let y1 = (x1 - startx) * m + starty;
			(y, y1)
		};
		let (y0, y1) = if y0 > y1 {(y1, y0)} else {(y0, y1)};
		let (starty, endy) = if starty > endy {(endy, starty)} else {(starty, endy)};
		let (y0, y1) = ((std::cmp::max_by(y0, starty, |a, b| a.partial_cmp(b).unwrap_or(Ordering::Less)) / 50f32) as u16, (std::cmp::min_by(y1, endy, |a, b| a.partial_cmp(b).unwrap_or(Ordering::Greater)) / 50f32) as u16);
		for y in y0..=y1 {
			match po.getCtx().getMap().getScreen(po.getCtx().getMap().getActiveScreenId()).unwrap().getTile((x, y)).getCollisionType() {
				CollisionType::Block => {
					return false;
				}
				CollisionType::OOB => {},
				_ => ()
			};
		}
		
	}
	true
}
