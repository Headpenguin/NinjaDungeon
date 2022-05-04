#![allow(non_snake_case)]
extern crate sdl2;

use NinjaDungeon::GameContext;

use sdl2::pixels::Color;
use sdl2::image::LoadTexture;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const NAME: &str = "test";

const COLOR: Color = Color::RGB(0x88, 0x88, 0x88);

fn main() 
{

	let (mut context, creator) = GameContext::initialize(NAME, WIDTH, HEIGHT, COLOR);
	
	let texture = creator.load_texture("Resources/Images/roundup-peoplekiller.png").unwrap();

	while context.mainLoop(&texture) {}

}

