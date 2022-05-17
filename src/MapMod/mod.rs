mod TileMod;
mod ScreenMod;

use sdl2::render::{TextureCreator, Canvas};
use sdl2::video::{Window, WindowContext};
use sdl2::rect::Rect;

use BinaryFileIO::BFStream::{ProvideReferencesDynamic, DynamicBinaryTranslator, ProvidePointersMutDynamic, DynamicTypedTranslator, SelfOwned};
use BinaryFileIO::BinaryDataContainer;

use std::io;
use std::ptr::addr_of_mut;

pub use TileMod::*;
pub use ScreenMod::*;

use crate::SpriteLoader::Animations;

const TILE_DIVISOR: f32 = 1f32/50f32;

pub struct Map<'a> {
	screens: Vec<Screen>,
	activeScreen: usize,
	renderer: TileRenderer<'a>,
}

pub struct CollisionBounds {
	startX: u16,
	endX: u16,
	endY: u16,
	x: u16,
	y: u16,
}

pub struct TileRenderer<'a> {
	animations: Animations<'a>,
}

impl<'a> Map<'a> {
	/*pub fn fromFile(filename: &str, tileset: &str, textureCreator: &'a TextureCreator) -> Map<'a> {
		
	}*/
	pub fn new(id: usize, tileset: &str, textureCreator: &'a TextureCreator<WindowContext>) -> io::Result<Map<'a>> {
		Ok(Map {
			screens: vec![],
			activeScreen: 0,
			renderer: TileRenderer::new(id, tileset, textureCreator)?,
		})
	}
	pub fn update(&mut self) {
		self.renderer.update();
	}
	pub fn draw(&mut self, canvas: &mut Canvas<Window>) {
		self.screens[self.activeScreen].draw(&mut self.renderer, canvas);
	}
	pub fn addScreen(&mut self, width: u16, height: u16, location: Location) {
		self.screens.push(Screen::new(width, height, location));
		self.activeScreen = self.screens.len() - 1;
	}
	pub fn changeTile(&mut self, position: (u16, u16), replacement: Tile) {
		self.screens[self.activeScreen].replaceTile(position, replacement);
	}
	pub fn renderTile(&mut self, position: Rect, tile: &Tile, canvas: &mut Canvas<Window>) {
		self.renderer.draw(tile, canvas, position);
	}
	pub fn incrementCurrentScreen(&mut self) {
		if self.activeScreen + 1 < self.screens.len() {self.activeScreen+=1;}
	}
	pub fn decrementCurrentScreen(&mut self) {
		if self.activeScreen > 0 {self.activeScreen-=1;}
	}
	pub fn setCurrentScreen(&mut self, screen: usize) -> Result<(), &'static str> {
		if screen < self.screens.len() {
			self.activeScreen = screen;
			Ok(())
		}
		else {Err("Attempted to switch to out-of-bounds screen")}
	}
	#[inline(always)]
	pub fn calculateCollisionBounds(hitbox: Rect) -> CollisionBounds {
		let leftBound = (hitbox.x as f32 * TILE_DIVISOR ).floor() as u16;
        let rightBound = ((hitbox.x + hitbox.w) as f32 * TILE_DIVISOR ).floor() as u16;
        let topBound = (hitbox.y as f32 * TILE_DIVISOR ).floor() as u16;
        let bottomBound = ((hitbox.y + hitbox.h) as f32 * TILE_DIVISOR ).floor() as u16;
		CollisionBounds	{
			startX: leftBound,
			endX: rightBound,
			endY: bottomBound,
			x: leftBound,
			y: topBound,
		}
	}
	pub fn collide(&'a self, bounds: &mut CollisionBounds) -> Option<&'a Tile> {
		Some(self.screens[self.activeScreen].getTile(bounds.next()?))
	}
	pub unsafe fn createRenderer(&mut self, tileset: &str, textureCreator: &'a TextureCreator<WindowContext>) {
		addr_of_mut!(self.renderer).write(TileRenderer::new(0, tileset, textureCreator).unwrap());
	}
}

impl Iterator for CollisionBounds {
	type Item = (u16, u16);
	fn next(&mut self) -> Option<Self::Item> {
		let result = Some((self.x, self.y));
		if self.y > self.endY {
			return None;
		}
		if self.x >= self.endX {
			self.x = self.startX;
			self.y += 1;
		}
		else{self.x += 1;}
		result
	}
}

unsafe impl<'a> SelfOwned for Map<'a> {}

impl<'a> ProvideReferencesDynamic<'a> for Map<'a> {
	type Type = Map<'static>;
	fn provideReferencesDyn<T: DynamicBinaryTranslator<'a>>(&'a self, translator: &mut T) {
		unsafe{translator.translateSlice(self.screens.as_slice())};
	}
}

impl<'a> ProvidePointersMutDynamic<'a> for Map<'a> {
	type Type = Map<'static>;
	unsafe fn providePointersMutDyn<T: DynamicTypedTranslator<'a>>(uninitialized: *mut Self, depth: usize, translator: &mut T) -> bool {
		if depth == 0 {
			let size = translator.getSliceSize().unwrap();
			let mut v = Vec::with_capacity(size);
			let ptr = v.as_mut_ptr();
			let translatedPtr: *mut [Screen] = BinaryDataContainer::reinterpretAllocatedToSlice(ptr as *mut u8, size);
			translator.translateRawSlice(translatedPtr);
			v.set_len(size);
			addr_of_mut!((*uninitialized).screens).write(v);
			false
		}
		else {
			translator.translateSlice(depth - 1, (*uninitialized).screens.as_mut_slice())
		}
	}
}

impl<'a> TileRenderer<'a> {
	pub fn new(id: usize, tileset: &str, creator: &'a TextureCreator<WindowContext>) -> io::Result<TileRenderer<'a>> {
		Ok(TileRenderer {
			animations: Animations::new(tileset, TILESETS[id], creator)?,
		})
	}
	pub fn update(&mut self) {
		self.animations.update();
	}
	// Make this better pls
	pub fn draw(&mut self, tile: &Tile, canvas: &mut Canvas<Window>, position: Rect) {
		self.animations.changeAnimation(tile.getId() as usize).unwrap();
		self.animations.drawNextFrame(canvas, position);
	}
}

const TILESETS: &'static [&'static [&'static str]] = &[
	&[
		"Ground",
		"Wall",
		"Gate",
	],
];

