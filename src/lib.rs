#![allow(non_snake_case)]
extern crate sdl2;
extern crate BinaryFileIO;

use sdl2::{Sdl, VideoSubsystem, EventPump};
use sdl2::render::{Canvas, TextureCreator, Texture};
use sdl2::video::{WindowContext, Window};
use sdl2::event::Event;
use sdl2::hint;
use sdl2::pixels::Color;
use sdl2::rect::{Rect, Point};
use sdl2::ttf::{Sdl2TtfContext, Font, self};
use sdl2::keyboard::{TextInputUtil, Scancode};
use sdl2::mouse::{MouseButton, MouseWheelDirection};

use BinaryFileIO::{load, dump};

use std::io;
use std::i32;
use std::str::FromStr;

mod PlayerMod;
mod SpriteLoader;
mod MapMod;
mod Vec2dMod;
mod VectorMod;
pub mod Entities;

pub use VectorMod::Vector;
pub use Vec2dMod::Vec2d;
pub use PlayerMod::Player;

pub use MapMod::{Map, Location, Tile, MAX_TILE_IDX, CollisionType, CollisionBounds, TileBuilder, TileBuilderSignals};

use PlayerMod::SignalsBuilder;

use Entities::Codes;

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
		player.update(map);

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

pub struct EditorContext {
	sdlContext: Sdl,
	videoSubsystem: VideoSubsystem,
	canvas: Canvas<Window>,
	events: EventPump,
	textInput: TextInputUtil,
	quit: bool,
    mapRes: (u32, u32),
    mapRect: Rect,
	currentTileId: u16,
	currentTilePosition: (u16, u16),
	currentTile: Tile,
	previewTile: Tile,
	previewRect: Rect,
	tileBuilder: TileBuilder,
	screenRect: Rect,
	state: State,
	lock: bool,
	message: String,
	messageLen: usize,
}

impl EditorContext {
	pub fn new(width: u32, height: u32, name: &'static str, color: Color,) -> (TextureCreator<WindowContext>, Sdl2TtfContext, EditorContext) {

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
		
		let ttfContext = ttf::init().unwrap();

		canvas.set_draw_color(color);

		(textureCreator, ttfContext, EditorContext {
			canvas,
			events: sdlContext.event_pump().unwrap(),
			textInput: videoSubsystem.text_input(),
			sdlContext,
			videoSubsystem,
			quit: false,
			currentTileId: 0,
			currentTilePosition: (0, 0),
			currentTile: Tile::new(0, 0).unwrap(),
			previewTile: Tile::new(0, 0).unwrap(),
			screenRect: Rect::new(0, 0, width, height),
            mapRes: (136, 104),
			mapRect: Rect::new(0, 0, width, height),
			previewRect: Rect::new(0, height as i32 - 50, 50, 50),
			tileBuilder: TileBuilder::new(0),
			state: State::Idle,
			lock: false,
			message: String::new(),
			messageLen: 0,
		})
	}

	pub fn mainLoop<'a>(&mut self, filename: &str, map: &mut Map, font: &Font, fontTexture: &mut Option<Texture<'a>>, textureCreator: &'a TextureCreator<WindowContext>) -> bool {
		self.canvas.clear();
	
		for event in self.events.poll_iter() {match &self.state {
			State::Idle => match event {
				Event::Quit {..} => self.quit = true,

				Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} 
				if (y as i64) < (self.screenRect.height() - 50) as i64 => {
					self.tileBuilder = TileBuilder::new(self.currentTileId);
					self.currentTilePosition = ((x / 50) as u16, (y / 50) as u16);
					self.state = State::AttemptBuild;
					break;
				},
				Event::MouseButtonDown {mouse_btn: MouseButton::Right, x, y, ..} => {
					self.currentTilePosition = ((x / 50) as u16, (y / 50) as u16);
					map.changeTile(self.currentTilePosition, self.tileBuilder.cloneTile(&self.currentTile));
				},
				Event::KeyDown{scancode: Some(Scancode::Left), ..} => {
					if self.currentTileId > 0 {
						self.currentTileId -= 1;
						self.previewTile = Tile::new(self.currentTileId, 0).unwrap();
					}
				},
				Event::KeyDown{scancode: Some(Scancode::Right), ..} => {
					if self.currentTileId < MAX_TILE_IDX {
						self.currentTileId += 1;
						self.previewTile = Tile::new(self.currentTileId, 0).unwrap();
					}
				},
				Event::KeyDown{scancode: Some(Scancode::S), ..} => {
					dump!(filename, *map).unwrap();
				},	
				Event::KeyDown{scancode: Some(Scancode::A), ..} => {
					map.decrementCurrentScreen();
				},
				Event::KeyDown{scancode: Some(Scancode::D), ..} => {
					map.incrementCurrentScreen();
				},
				Event::KeyDown{scancode: Some(Scancode::Escape), ..} => {
					self.state = State::ViewMap;
				}
				_ => (),
			},
			State::ViewMap => match event {
				Event::Quit {..} => self.quit = true,
                Event::KeyDown {scancode: Some(Scancode::Escape), ..} => {
                    self.state = State::Idle;
                }
				Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} => {
                    let (x, y) = MapMod::convertScreenCoordToTileCoord(self.mapRes, self.mapRect, Point::from((x, y))).into();
					println!("{}, {}", x, y);
                    map.addScreen(17, 12, (x as u32, y as u32));
					self.state = State::Idle;
                },
				Event::MouseWheel {y: 1, ..} => {
					if self.mapRes.0 > 1 && self.mapRes.1 > 1 {
						self.mapRes.0 /= 2;
						self.mapRes.1 /= 2;
					}
				},
				Event::MouseWheel {y: -1, ..} => {
					if self.mapRes.0 < i32::MAX as u32 && self.mapRes.1 < i32::MAX as u32 {
						self.mapRes.0 *= 2;
						self.mapRes.1 *= 2;
					}
				}
				Event::KeyDown {scancode: Some(Scancode::H), ..} => self.mapRect.offset(((self.mapRes.0 >> 1) as i32).clamp(0, self.mapRect.x()) * -1, 0),
				Event::KeyDown {scancode: Some(Scancode::J), ..} => self.mapRect.offset(0, ((self.mapRes.1 >> 1) as i32).clamp(0, self.mapRect.y()) * -1),
				Event::KeyDown {scancode: Some(Scancode::K), ..} => self.mapRect.offset(0, ((self.mapRes.1 >> 1) as i32).clamp(0, i32::MAX - self.mapRect.y())),
				Event::KeyDown {scancode: Some(Scancode::L), ..} => self.mapRect.offset(((self.mapRes.0 >> 1) as i32).clamp(0, i32::MAX - self.mapRect.x()), 0),
				_ => (),
			}
			_ => match (event, &self.state) {
				(Event::Quit {..}, _) => self.quit = true,
				(_, State::Idle) => self.lock = false,
				(Event::KeyDown {scancode: Some(Scancode::Escape), ..}, _) => {
					self.lock = false;
					self.state = State::Idle;
					*fontTexture = None;
				}
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::GetUserUsize) => {
					if let Ok(id) = usize::from_str(&self.message[self.messageLen..].trim()) {
						self.lock = false;
						self.tileBuilder.addUsize(id);
						self.state = State::AttemptBuild;
						*fontTexture = None;
					}
					else {
						self.message.truncate(self.messageLen);
						*fontTexture = Some(font.render(&self.message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(&textureCreator).unwrap());
					}
				},
				(Event::TextInput {text, ..}, _) => {
					self.message.push_str(&text);
					*fontTexture = Some(font.render(&self.message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(&textureCreator).unwrap());
				},
				(Event::KeyDown {scancode: Some(Scancode::Backspace), ..}, _) if self.message.len() > self.messageLen => {
					self.message.pop();
					*fontTexture = Some(font.render(&self.message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(&textureCreator).unwrap());
				},
				_ => (),
			},
		}}

		match self.state {
			State::AttemptBuild => self.build(map, font, fontTexture, textureCreator),
			_ => (),
		}

		match self.state {
			State::ViewMap => map.drawAll(&mut self.canvas, self.mapRes, self.mapRect),
			_ => {
				map.draw(&mut self.canvas);
				map.renderTile(self.previewRect, &self.previewTile, &mut self.canvas);
			},
		}
		if let Some(ref texture) = fontTexture {
			let q = texture.query();
			self.canvas.copy(texture, None, Some(Rect::from_center(self.screenRect.center(), q.width, q.height)));
		}
		
		self.canvas.present();

		self.quit
	}
	fn build<'a>(&mut self, map: &mut Map, font: &Font, fontTexture: &mut Option<Texture<'a>>, textureCreator: &'a TextureCreator<WindowContext>) {
		match self.tileBuilder.build() {
			TileBuilderSignals::GetUserUsize(tmpMessage) => {
				self.state = State::GetUserUsize;
				self.lock = true;
				self.message = String::from(tmpMessage);
				self.messageLen = tmpMessage.len() - 1;
				*fontTexture = Some(font.render(&self.message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(&textureCreator).unwrap());
			},
			TileBuilderSignals::Complete(tile) => {
				self.state = State::Idle;
				self.currentTile = tile;
				map.changeTile(self.currentTilePosition, self.tileBuilder.cloneTile(&self.currentTile));
			},
			TileBuilderSignals::InvalidId => (),
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

pub trait PlayerCollision {
    fn collidePlayer(&self, player: &mut Player);
}

pub struct Entity {
	code: Codes,
	position: (i32, i32),
	direction: Direction,
}

enum State {
	GetUserUsize,
	ViewMap,
    NewMap,
	AttemptBuild,
	Idle,
}

