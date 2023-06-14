#![allow(non_snake_case)]
extern crate sdl2;
extern crate serde;
extern crate serde_json;
extern crate rand;

use sdl2::{Sdl, VideoSubsystem, EventPump};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{WindowContext, Window};
use sdl2::event::Event;
use sdl2::hint;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Texture;

use serde::{Serialize, Deserialize};

use serde_json::Deserializer;

use std::io;
use std::fs::File;
use std::cell::UnsafeCell;

mod PlayerMod;
mod SpriteLoader;
mod MapMod;
mod Vec2dMod;
mod VectorMod;
mod IntHasher;
mod EventProcessor;
mod Scheduling;
mod GameContextMod;
pub mod Entities;
mod EditorContextMod;

pub use VectorMod::Vector;
pub use Vec2dMod::Vec2d;
pub use PlayerMod::Player;

pub use MapMod::*;
pub use GameContextMod::*;
pub use EditorContextMod::{EditorContext, EditorContextDeps, createText};

use PlayerMod::SignalsBuilder;

pub use EventProcessor::PO;

use Scheduling::Scheduler;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ID(u64);

const ID_MASK: u64 = 0xffffffffffffff00;

impl ID {
	pub fn new(id: u64, subID: u8) -> ID {
		ID((id << 8) + subID as u64)
	}
	pub fn sub(&self, subID: u8) -> ID {
		ID((self.0 & ID_MASK) + subID as u64)
	}
	pub fn empty() -> ID {
		ID(u64::MAX)
	}
	pub fn getID(&self) -> u64 {
		(self.0 & ID_MASK) >> 8
	}
	pub fn getSubID(&self) -> u8 {
		(self.0 & 0xff) as u8
	}
	pub fn isEmpty(&self) -> bool {
		self.0 == u64::MAX
	}
	pub fn mask(&self) -> ID {
		ID(self.0 & ID_MASK)
	}
}

pub struct GameManager {
	sdlContext: Sdl,
	videoSubsystem: VideoSubsystem,
	canvas: Canvas<Window>,
	events: EventPump,
	screenPos: Point,
	scheduler: Scheduler,
	quit: bool,
	pub advance: bool,
}

impl GameManager {
	
	pub fn initialize(name: &'static str, width: u32, height: u32, color: Color) -> (GameManager, TextureCreator<WindowContext>) {
		
		let sdlContext = sdl2::init().unwrap();
		let videoSubsystem = sdlContext.video().unwrap();
		
		if !hint::set("SDL_RENDER_SCALE_QUALITY", "1") {
		
			eprintln!("Warning: Linear texture filtering may not be enabled.");
		
		}

		let window = videoSubsystem.window(name, width, height)
			.position_centered()
			.build()
			.unwrap();
		
		let mut canvas = window.into_canvas()
			.present_vsync()
			.build()
			.unwrap();
	
		let textureCreator = canvas.texture_creator();

		let events = sdlContext.event_pump().unwrap();
		
		canvas.set_draw_color(color);

		let quit = false;
		let screenPos = Point::new(0, 0);

		(GameManager {sdlContext, videoSubsystem, canvas, events, screenPos, quit, scheduler: Scheduler::new(), advance: false}, textureCreator,) 
	}
	
	#[inline(always)]
	pub fn mainLoop<'a, 'b>(&mut self, po: &'b mut UnsafeCell<PO<'a>>) -> bool {
		
		self.canvas.clear();
		
		let mut signals = SignalsBuilder::default();

		for event in self.events.poll_iter() {
			self.quit = Self::windowEvents(&event);
			signals.addEvent(&event);
		}
		
		let ctx = unsafe {po.get_mut().getCtxMut()};
		let player = ctx.holder.getMutTyped(ctx.getPlayerID()).unwrap();
		player.signal(signals.build(&self.events));
//		player.transition(&mut po.getCtx()map);
//		po.get_mut().transition();

		unsafe {
			if (&mut *po.get()).getCtxMut().getPlayerMut().transition((&mut *po.get()).getCtxMut()) {
				//println!("dgf");
				//po.get_mut().getCtxMut().resetCollisionLists();
				po.get_mut().getCtxMut().disableEntityCollisionFrame();
			}
		}
		

		unsafe {
			Scheduler::tick(po.get_mut().getCtxMut());
			self.scheduler.execute(po, |id| {(&mut *(&*po.get()).getCtx().getHolder().getEntityDyn(id).unwrap()).getData(&*po.get(), EventProcessor::Key::new());});
			po.get_mut().getCtxMut().resetCollisionLists();
			self.scheduler.execute(po, |id| (&mut *(&*po.get()).getCtx().getHolder().getEntityDyn(id).unwrap()).update(&mut *po.get()) );
		}
		unsafe {
			po.get_mut().getCtxMut().map.update();

			match po.get_mut().doCommands() {
				1 => {
					self.quit = true;
					self.advance = true;
				},
				2 => {
					self.quit = true;
					self.advance = false;
				},
				_ => (),
			}

			po.get_mut().getCtxMut().map.draw(&mut self.canvas, self.screenPos);
		}
		unsafe { self.scheduler.draw(po.get_mut().getCtx(), &mut self.canvas) ;}
		unsafe {po.get_mut().purge();}
		
		self.canvas.present();
		
		!self.quit
	}

	pub fn conglaturate(&mut self, conglaturate: &Texture) -> bool {
		self.quit = false;
		self.canvas.clear();
		for event in self.events.poll_iter() {
			self.quit = Self::windowEvents(&event);
		}
		self.canvas.copy(conglaturate, None, Rect::new(0, 0, 250, 50));
		
		self.canvas.present();
		
		!self.quit
		
	}

	fn windowEvents(event: &Event) -> bool {
		match event {
			Event::Quit{..} => true,
			_ => false,
		}
	}
}

pub fn loadCtx<'a>(filename: &str, creator: &'a TextureCreator<WindowContext>) -> io::Result<GameContext<'a>> {
	let mut deserializer = Deserializer::from_reader(File::open(filename)?);
	let ctx = InnerGameContext::deserialize(&mut deserializer)?;

	ctx.intoGameContext(creator)
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Direction {
	Up,
	Down,
	Left,
	Right,
}

impl Default for Direction {
	fn default() -> Self {
		Direction::Up
	}
}

