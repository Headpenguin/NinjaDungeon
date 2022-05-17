#![allow(non_snake_case)]
extern crate sdl2;

use NinjaDungeon::{GameContext, Player, loadMap};

use sdl2::pixels::Color;
use sdl2::image::LoadTexture;

const WIDTH: u32 = 17*50;
const HEIGHT: u32 = 12*50;

const NAME: &str = "test";

const COLOR: Color = Color::RGB(0x88, 0x88, 0x88);

fn main() 
{

	let (mut context, creator) = GameContext::initialize(NAME, WIDTH, HEIGHT, COLOR);
	
	let mut player = Player::new(&creator, 50f32, 50f32).unwrap();

	let mut map = loadMap("Resources/Levels/urMom.mp", "Resources/Images/Map1.anim", &creator).unwrap();

	while context.mainLoop(&mut player, &mut map) {}

}

