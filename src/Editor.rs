#![allow(non_snake_case)]
extern crate sdl2;

use NinjaDungeon::{Map, EditorContext, EditorContextDeps, GameContext, self};
use NinjaDungeon::Entities::EntityRenderer;

use sdl2::pixels::Color;


use std::env;
use std::fs;

const DEFAULT_LOOKUP: &str = "Resources/MapName.txt";

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
fn main() {
	let (textureCreator, ttfContext, mut events, mut editor) = EditorContext::new(WIDTH, HEIGHT, NAME, COLOR);

	let file = match env::args().skip(1).next() {
		Some(name) => name,
		None => fs::read_to_string(DEFAULT_LOOKUP).unwrap_or_else(|_| {
			eprintln!("Warning: Could not find any map filenames!");
			String::from("map.out")
		}),
	};

	let mut ctx = NinjaDungeon::loadCtx(&file, &textureCreator).unwrap_or_else(|_| {
		eprintln!("Warning: Could not read map file \"{}\"", &file);
		let mut map = Map::new(0, "Resources/Images/Map1.anim", &textureCreator).unwrap();
		map.addScreen(17, 12, (0, 0));
		GameContext::new(map, &textureCreator)
	});

	let font = ttfContext.load_font("Resources/Font/Symbola_hint.ttf", 16).unwrap();

	let mut fontTexture = None;

	let mut idTexture = Some(NinjaDungeon::createText(&ctx.getMap().getActiveScreenId().to_string(), &textureCreator, &font));

	let entityRenderer = EntityRenderer::new(&textureCreator).unwrap();

	//let mut entities = Vec::<Entity>::new();

    let mut editorContextDeps = EditorContextDeps{
        filename: &file,
        font: &font,
        fontTexture: &mut fontTexture,
        idTexture: &mut idTexture,
        entityRenderer: &entityRenderer,
        textureCreator: &textureCreator,
        ctx: &mut ctx,
    };
	
	while !editor.mainLoop(&mut events, &mut editorContextDeps) {
	}
}

