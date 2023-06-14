#![allow(non_snake_case)]
extern crate sdl2;

use NinjaDungeon::{GameManager, GameContext, Player, loadCtx};
use NinjaDungeon::PO;

use sdl2::pixels::Color;
use sdl2::image::LoadTexture;
use sdl2::render::Texture;

use std::fs;

use std::cell::UnsafeCell;

const WIDTH: u32 = 17*50;
const HEIGHT: u32 = 12*50;

const NAME: &str = "Ninja Dungeon";

const COLOR: Color = Color::RGB(0x88, 0x88, 0x88);

fn main() 
{

	let (mut manager, creator) = GameManager::initialize(NAME, WIDTH, HEIGHT, COLOR);

	let mut currentMap = match fs::read_to_string("Resources/CurrentMap").unwrap().trim() {
		"0" => 0,
		"1" => 1,
		_ => panic!("Too high of a map id"),
	};

	
	let ctx = if currentMap == 0 {loadCtx("Resources/Map1.mp", &creator).unwrap()}
	else {loadCtx("Resources/Map2.mp", &creator).unwrap()};

	let mut po = UnsafeCell::new(PO::new(ctx));

	while manager.mainLoop(&mut po) {}
	if manager.advance {
		fs::write("Resources/CurrentMap", if currentMap == 0 {"1"} else {"0"}).unwrap();
	let conglaturations = creator.load_texture("Resources/Images/Conglaturations.png").unwrap();
	while manager.conglaturate(&conglaturations) {}
	}

}

