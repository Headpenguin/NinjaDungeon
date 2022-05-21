#![allow(non_snake_case)]
extern crate BinaryFileIO;
extern crate sdl2;

use NinjaDungeon::{Map, Location, Tile, Direction, MAX_TILE_IDX, TileBuilder, TileBuilderSignals};
use NinjaDungeon::Entities::Codes;

use sdl2::hint;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Scancode;
use sdl2::rect::Rect;
use sdl2::ttf::{self, Font};
use sdl2::render::{TextureCreator, Texture};
use sdl2::video::WindowContext;

use BinaryFileIO::{load, dump};

use std::io;
use std::env;
use std::fs;
use std::str::FromStr;

const DEFAULT_LOOKUP: &str = "MapName.txt";

const WIDTH: u32 = 17*50;
const HEIGHT: u32 = 13*50;
const NAME: &str = "test";
const COLOR: Color = Color::RGB(0x88, 0x88, 0x88);

/*pub fn addEntity(&mut self, code: Codes, position: (i32, i32), direction: Direction) {
	self.entities.push(Entity{code, position, direction});
}
pub fn removeEntity(&mut self, position: (i32, i32)) -> Result<(), ()> {
	self.entities.remove(self.entities.iter().position(|e| e.position == position).ok_or(())?);
	Ok(())
}*/
struct Entity {
	code: Codes,
	position: (i32, i32),
	direction: Direction,
}
fn main() {    
	let SCREEN_RECT: Rect = Rect::new(0, 0, WIDTH, HEIGHT);
	let file = match env::args().skip(1).next() {
		Some(name) => name,
		None => fs::read_to_string(DEFAULT_LOOKUP).unwrap_or_else(|_| {
			println!("Warning: Could not find backup map file!");
			String::from("")
		}),
	};

	let sdlContext = sdl2::init().unwrap();
	let videoSubsystem = sdlContext.video().unwrap();
	
	if !hint::set("SDL_RENDER_SCALE_QUALITY", "1") {
	
		eprintln!("Warning: Linear texture filtering may not be enabled.");
	
	}

	let window = videoSubsystem.window(NAME, WIDTH, HEIGHT)
		.position_centered()
		.build()
		.unwrap();
	
	let mut canvas = window.into_canvas()
		.present_vsync()
		.build()
		.unwrap();

	let textureCreator = canvas.texture_creator();

	let ttfContext = ttf::init().unwrap();

	let font = ttfContext.load_font("Resources/Font/Symbola_hint.ttf", 16).unwrap();

	let mut fontTexture = None;

	let textInput = videoSubsystem.text_input();

	let map: io::Result<(Map,)> = unsafe{load!(&file, map)};

	let mut map: Map = match map {
		Ok((mut map,)) => unsafe {map.createRenderer("Resources/Images/Map1.anim", &textureCreator); map},
		Err(..) => {
			println!("Warning: Could not read map file \"{}\"", &file);
			let mut map = Map::new(0, "Resources/Images/Map1.anim", &textureCreator).unwrap();
			map.addScreen(17, 12, (0, 0));
			map
		},
	};

	let mut entities = Vec::<Entity>::new();

	let mut currentTileId = 0u16;

	let mut currentTile = Tile::new(currentTileId, 0).unwrap();

	let tileRect = Rect::new(0, HEIGHT as i32 - 50, 50, 50);

	let mut events = sdlContext.event_pump().unwrap();

	let mut builder = TileBuilder::new(currentTileId);

	let mut state = State::Idle;

	let mut lock = false;

	let mut message = String::from("");

	let mut messageLen = 0;

	canvas.set_draw_color(COLOR);

	let mut quit = false;
	
	while !quit {

		canvas.clear();
	
		for event in events.poll_iter() {
			if !lock {match event {
				Event::Quit {..} => quit = true,

				Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} 
				if (y as i64) < (HEIGHT - 50) as i64 => {
					builder = TileBuilder::new(currentTileId);
					buil
				},

				Event::KeyDown{scancode: Some(Scancode::Left), ..} => {
					if currentTileId > 0 {
						currentTileId -= 1;
						currentTile = Tile::new(currentTileId, 0).unwrap();
					}
				},
				Event::KeyDown{scancode: Some(Scancode::Right), ..} => {
					if currentTileId < MAX_TILE_IDX {
						currentTileId += 1;
						currentTile = Tile::new(currentTileId, 0).unwrap();
					}
				},
				Event::KeyDown{scancode: Some(Scancode::S), ..} => {
					dump!(&file, map).unwrap();
				},	
				Event::KeyDown{scancode: Some(Scancode::N), ..} => {
					map.addScreen(17, 12, (0, 0));
				},
				Event::KeyDown{scancode: Some(Scancode::A), ..} => {
					map.decrementCurrentScreen();
				},
				Event::KeyDown{scancode: Some(Scancode::D), ..} => {
					map.incrementCurrentScreen();
				},
				_ => (),
			}}
			else {match (event, &state) {
				(Event::Quit {..}, _) => quit = true,
				(_, State::Idle) => lock = false,
				(Event::KeyDown {scancode: Some(Scancode::Escape), ..}, _) => {
					lock = false;
					state = State::Idle;
					fontTexture = None;
				}
				(Event::KeyDown {scancode: Some(Scancode::Return), ..}, State::GetUserUsize) => {
					println!("{}", &message[messageLen..]);
					if let Ok(id) = usize::from_str(&message[messageLen..].trim()) {
						lock = false;
						builder.addUsize(id);
						state = State::AttemptBuild;
						fontTexture = None;
					}
					else {
						message.truncate(messageLen);
						fontTexture = Some(font.render(&message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(&textureCreator).unwrap());
					}
				},
				(Event::TextInput {text, ..}, _) => {
					message.push_str(&text);
					fontTexture = Some(font.render(&message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(&textureCreator).unwrap());
				},
				(Event::KeyDown {scancode: Some(Scancode::Backspace), ..}, _) if message.len() > messageLen => {
					message.pop();
					fontTexture = Some(font.render(&message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(&textureCreator).unwrap());
				},
				_ => (),
			}}
		}

		map.draw(&mut canvas);

		if let Some(ref texture) = fontTexture {
			let q = texture.query();
			canvas.copy(texture, None, Some(Rect::from_center(SCREEN_RECT.center(), q.width, q.height)));
		}

		map.renderTile(tileRect, &currentTile, &mut canvas);
		
		canvas.present();

	}
}

fn build<'a>(builder: &mut TileBuilder, map: &mut Map, state: &mut State, lock: &mut bool, 
	message: &mut String, messageLen: &mut usize, fontTexture: &'a mut Option<Texture<'a>>, 
	font: &Font, textureCreator: &'a TextureCreator<WindowContext>, x: &i32, y: &i32) {
	match builder.build() {
		TileBuilderSignals::GetUserUsize(tmpMessage) => {
			*state = State::GetUserUsize;
			*lock = true;
			*message = String::from(tmpMessage);
			*messageLen = tmpMessage.len() - 1;
			*fontTexture = Some(font.render(&message).shaded(Color::BLACK, Color::WHITE).unwrap().as_texture(&textureCreator).unwrap());
		},
		TileBuilderSignals::Complete(tile) => {
			*state = State::Idle;
			map.changeTile(((x / 50) as u16, (y / 50) as u16), tile);
		},
		TileBuilderSignals::InvalidId => (),
	}	
}

enum State {
	GetUserUsize,
	AttemptBuild,
	Idle,
}

