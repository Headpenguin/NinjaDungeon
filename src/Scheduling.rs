use sdl2::render::Canvas;
use sdl2::video::Window;
use std::iter::Iterator;
use std::cell::UnsafeCell;
use crate::{GameContext, ID, PO};
pub struct Scheduler{}
impl Scheduler {
	pub fn new() -> Scheduler {Scheduler{}}
	pub unsafe fn execute<'a, 'b: 'a, F>(&self, po: &UnsafeCell<PO<'b>>, f: F) where
	F: FnMut(ID) {
		(&mut *po.get()).getCtx().entityIter().filter_map(|kv| {
			if kv.1.needsExecution() {
				Some(kv.0)
			}
			else {
				None
			}
		}).for_each(f);
	}
	pub unsafe fn tick(ctx: &mut GameContext) {
		for e in ctx.entityIterMut() {
			e.1.tick();
		}
	}
	pub unsafe fn draw<'a, 'b: 'a>(&self, ctx: &'a GameContext<'b>, canvas: &mut Canvas<Window>) {
		for e in ctx.entityIter() {
			e.1.draw(canvas);
		}
	}
}
