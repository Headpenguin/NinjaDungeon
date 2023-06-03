use sdl2::render::Canvas;
use sdl2::video::Window;
use std::iter::Iterator;
use std::cell::UnsafeCell;
use crate::{GameContext, ID, PO};
pub struct Scheduler{}
impl Scheduler {
	pub fn new() -> Scheduler {Scheduler{}}
	pub unsafe fn execute<'a, 'b: 'a, F>(&self, po: &UnsafeCell<PO<'b>>, mut f: F) where
	F: FnMut(ID) {
		for id in (&mut *po.get()).getCtx().activeScreenEntityIter().filter_map(|id| {
			if (&mut *(&mut *po.get()).getCtx().getHolder().getEntityDyn(id).unwrap()).needsExecution() {
				Some(id)
			}
			else {
				None
			}
		}) {
			f(id);
		}
		for id in (&mut *po.get()).getCtx().globalEntityIter().filter_map(|id| {
			if (&mut *(&mut *po.get()).getCtx().getHolder().getEntityDyn(id).unwrap()).needsExecution() {
				Some(id)
			}
			else {
				None
			}
		}) {
			f(id);
		}
	}
	pub unsafe fn tick(ctx: &mut GameContext) {
		for e in ctx.entityIterMut() {
			e.1.tick();
		}
	}
	pub unsafe fn draw<'a, 'b: 'a>(&self, ctx: &'a GameContext<'b>, canvas: &mut Canvas<Window>) {
		for id in ctx.activeScreenEntityIter() {
			(&mut *ctx.getHolder().getEntityDyn(id).unwrap()).draw(canvas);
		}
		for id in ctx.globalEntityIter() {
			(&mut *ctx.getHolder().getEntityDyn(id).unwrap()).draw(canvas);
		}
	}
}
