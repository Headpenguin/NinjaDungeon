#![allow(non_snake_case)]
extern crate BinaryFileIO;
extern crate sdl2;

use NinjaDungeon::{Map, Location, Tile};

use sdl2::hint;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;

const WIDTH: u32 = 17*50;
const HEIGHT: u32 = 13*50;
const NAME: &str = "test";
const COLOR: Color = Color::RGB(0x88, 0x88, 0x88);

fn main() {    
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

	let mut map = Map::new(0, 0, "Resources/Images/Map1.anim", &textureCreator).unwrap();

	map.addScreen(17, 12, Location::default());

	map.changeTile((0, 0), Tile::new(1, 0).unwrap());

	let mut events = sdlContext.event_pump().unwrap();
	
	canvas.set_draw_color(COLOR);

	let mut quit = false;

	while !quit {	

		canvas.clear();
	
		for event in events.poll_iter() {
			match event {
				Event::Quit {..} => quit = true,
				Event::MouseButtonDown {mouse_btn: MouseButton::Left, x, y, ..} => {
					if (y as i64) < (HEIGHT - 50) as i64 {
						map.changeTile(((x / 50) as u16, (y / 50) as u16), Tile::new(1, 0).unwrap());
					}
				}
				_ => (),
			}
		}

		map.draw(&mut canvas);
		
		canvas.present();

	}
}

