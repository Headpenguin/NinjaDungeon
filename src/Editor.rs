#![allow(non_snake_case)]
extern crate BinaryFileIO;

fn main() {    
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
}

