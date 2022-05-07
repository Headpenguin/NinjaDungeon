extern crate sdl2;

use std::default::Default;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;

pub struct SignalsBuilder {
	event: bool,
	mapping: Mapping,
}

pub struct Signals {
	pub up: Option<bool>,
	pub down: Option<bool>,
	pub left: Option<bool>,
	pub right: Option<bool>,
	pub attack: Option<bool>,
}

pub struct Mapping {	
	pub up: Scancode,
	pub down: Scancode,
	pub left: Scancode,
	pub right: Scancode,
	pub attack: Scancode,
}

impl SignalsBuilder {
	pub fn new(mapping: Mapping) -> SignalsBuilder {
		SignalsBuilder{event: false, mapping}
	}
	pub fn addEvent(&mut self, event: &Event) {
		if let Event::KeyDown{..} | Event::KeyUp{..} = event {
			self.event = true;
		}
	}
	pub fn build(self, events: &EventPump) -> Signals {
		if self.event {
			let state = events.keyboard_state();
			Signals {
				up: Some(state.is_scancode_pressed(self.mapping.up)),
				down: Some(state.is_scancode_pressed(self.mapping.down)),
				left: Some(state.is_scancode_pressed(self.mapping.left)),
				right: Some(state.is_scancode_pressed(self.mapping.right)),
				attack: Some(state.is_scancode_pressed(self.mapping.attack)),
			}
		}
		else {
			Signals {
				up: None,
				down: None,
				left: None,
				right: None,
				attack: None,	
			}
		}
	}
}

impl Default for SignalsBuilder {
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

