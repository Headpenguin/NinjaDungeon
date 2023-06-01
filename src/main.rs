#![allow(non_snake_case)]
extern crate sdl2;

use NinjaDungeon::{GameManager, GameContext, Player, loadMap};
use NinjaDungeon::PO;

use sdl2::pixels::Color;
use sdl2::image::LoadTexture;

const WIDTH: u32 = 17*50;
const HEIGHT: u32 = 12*50;

const NAME: &str = "test";

const COLOR: Color = Color::RGB(0x88, 0x88, 0x88);

fn main() 
{

	let (mut manager, creator) = GameManager::initialize(NAME, WIDTH, HEIGHT, COLOR);
	
	let mut map = loadMap("Resources/test2.mp", "Resources/Images/Map1.anim", &creator).unwrap();

	let mut po = PO::new(GameContext::new(map, &creator));

	while manager.mainLoop(&mut po) {}

}

