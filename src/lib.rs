#![allow(non_snake_case)]
extern crate sdl2;
extern crate BinaryFileIO;

use sdl2::{Sdl, VideoSubsystem, EventPump};
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{WindowContext, Window};
use sdl2::event::Event;
use sdl2::hint;
use sdl2::pixels::Color;

use BinaryFileIO::load;

use std::io;

mod PlayerMod;
mod SpriteLoader;
mod MapMod;
mod Vec2dMod;
pub mod Entities;

pub use Vec2dMod::Vec2d;
pub use PlayerMod::Player;

pub use MapMod::*;

use PlayerMod::SignalsBuilder;

pub struct GameContext {
	sdlContext: Sdl,
	videoSubsystem: VideoSubsystem,
	canvas: Canvas<Window>,
	events: EventPump,
	scriptPlayerInputs: bool,
	quit: bool,
}

impl GameContext {
	
	pub fn initialize(name: &'static str, width: u32, height: u32, color: Color) -> (GameContext, TextureCreator<WindowContext>) {
		
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

		let (scriptPlayerInputs, quit) = (false, false);

		(GameContext {sdlContext, videoSubsystem, canvas, events, scriptPlayerInputs, quit,}, textureCreator) 
	}
	
	#[inline(always)]
	pub fn mainLoop(&mut self, player: &mut Player, map: &mut Map) -> bool {
			
		self.canvas.clear();
		
		let mut signals = SignalsBuilder::default();

		for event in self.events.poll_iter() {
			self.quit = Self::windowEvents(&event);
			if !self.scriptPlayerInputs {
				signals.addEvent(&event);
			}
		}

		if !self.scriptPlayerInputs {
			player.signal(signals.build(&self.events));
		}

		map.update();
		player.update();

		map.draw(&mut self.canvas);
		player.draw(&mut self.canvas);
		
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

pub fn loadMap<'a>(filename: &str, tileSprites: &str, creator: &'a TextureCreator<WindowContext>) -> io::Result<Map<'a>> {
	let map: io::Result<(Map,)> = unsafe{load!(filename, map)};

	match map {
		Ok((mut map,)) => Ok(unsafe {map.createRenderer(tileSprites, &creator); map}),
		Err(e) => Err(e),
	}
}

#[repr(u8)]
pub enum Direction {
	Up,
	Down,
	Left,
	Right,
}

