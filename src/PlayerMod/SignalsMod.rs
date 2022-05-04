extern crate sdl2;

use std::default::Default;

use sdl2::event::Event;
use sdl2::keyboard::Scancode;

pub struct Signals {
	pub up: Option<bool>,
	pub down: Option<bool>,
	pub left: Option<bool>,
	pub right: Option<bool>,
	pub attack: Option<bool>,
	mapping: Mapping,
}

pub struct Mapping {	
	pub up: Scancode,
	pub down: Scancode,
	pub left: Scancode,
	pub right: Scancode,
	pub attack: Scancode,
}

impl Signals {
    pub fn new(mapping: Mapping) -> Signals {
		Signals {
			up: None,
			down: None,
			left: None,
			right: None,
			attack: None,
			mapping,
		}

    }
	pub fn addEvent(&mut self, event: &Event) {
		match event {
			Event::KeyDown {scancode: Some(scancode), ..} => {
				self.evaluateScancode(scancode, true);
			},
			Event::KeyUp {scancode: Some(scancode), ..} => {
				self.evaluateScancode(scancode, false);
			},
			_ => (),
		}
	}

	fn evaluateScancode(&mut self, scancode: &Scancode, pressOrRelease: bool) {
		let mapping = &self.mapping;
		if scancode == &mapping.up {self.up = Some(pressOrRelease)}
		if scancode == &mapping.down {self.down = Some(pressOrRelease)}
		if scancode == &mapping.left {self.left = Some(pressOrRelease)}
		if scancode == &mapping.right {self.right = Some(pressOrRelease)}
		if scancode == &mapping.attack {self.attack = Some(pressOrRelease)}
	}
}

impl Default for Signals {
	fn default() -> Self {
	    Self::new(Mapping::default())
    }
}

impl Default for Mapping {
	fn default() -> Self {
		Mapping {
			up: Scancode::Up,
			down: Scancode::Down,
			left: Scancode::Left,
			right: Scancode::Right,
			attack: Scancode::Space,
		}
	}
}

