#![allow(non_snake_case)]
extern crate sdl2;

use NinjaDungeon::{GameContext, Player};

use sdl2::pixels::Color;
use sdl2::image::LoadTexture;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 480;

const NAME: &str = "test";

const COLOR: Color = Color::RGB(0x88, 0x88, 0x88);

fn main() 
{

	let (mut context, creator) = GameContext::initialize(NAME, WIDTH, HEIGHT, COLOR);
	
	let mut player = Player::new(&creator, 0f32, 0f32).unwrap();

	while context.mainLoop(&mut player) {}

}

