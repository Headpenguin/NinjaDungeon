use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use std::io;

use super::Traits::{Collision, EntityTraitsWrappable, Entity};
use super::{BoxCode, RefCode, RefCodeMut};
use crate::SpriteLoader::Animations;
use crate::{GameContext, Vector};
use crate::EventProcessor::{CollisionMsg, Envelope, PO};


const NAMES_TOP: &'static[&'static str] = &[
	"Skeleton",
];

const NAMES_BOTTOM: &'static[&'static str] = &[
	"Skeleton idle",
	"Skeleton walk 0",
	"Skeleton walk 1",
];

enum ANIMATION_IDX_TOP {
	Skeleton = 0,
}

enum ANIMATION_IDX_BOTTOM {
	Idle = 0,
	Walk0,
	Walk1,
}

pub struct Skeleton<'a> {
	animationsTop: Animations<'a>,
	animationsBottom: Animations<'a>,
	timer: u32,
	renderPositionTop: Rect,
	renderPositionBottom: Rect,
	position: Vector,
	idle: bool,
//	iframeCounter: u32,
//	health
}

pub struct SkeletonData {
	nextPos: Vector,
}

impl<'a> Skeleton<'a> {
	pub fn new(creator: &'a TextureCreator<WindowContext>, position: (f32, f32)) -> io::Result<BoxCode<'a>> {
		let (timer, position, idle) = (
			0u32,
			Vector(position.0, position.1),
			true,
		);

		let animationsTop = Animations::new("Resources/Images/Skeleton_top.anim", NAMES_TOP, creator)?;
		let animationsBottom = Animations::new("Resources/Images/Skeleton_bottom.anim", NAMES_BOTTOM, creator)?;
		let renderPositionTop = Rect::new(position.0.round() as i32, position.1.round() as i32, 50, 50);
		let renderPositionBottom = Rect::new(position.0.round() as i32, position.1.round() as i32 + 50, 50, 50);
		Ok(BoxCode::Skeleton(
			Box::new(
				Entity::new(
					Skeleton {animationsTop, animationsBottom, timer, renderPositionTop, renderPositionBottom, position, idle},
					SkeletonData{
						nextPos: Vector(0f32, 0f32),
					},
				)
			)
		))
	}
	fn updatePositions(&mut self) {
		self.renderPositionTop.reposition(self.position);
		self.renderPositionBottom.reposition(self.position);
		// hitbox
	}
}

impl<'a> Collision for Skeleton<'a> {
	fn collide(&mut self, msg: Envelope<CollisionMsg>) {}
}

impl<'a> EntityTraitsWrappable<'a> for Skeleton<'a> {
	type Data = SkeletonData;
	fn mapCodeMut(code: RefCodeMut<'a>) -> Option<&'a mut Self> {
		if let RefCodeMut::Skeleton(s) = code {Some(s as &mut Self)}
		else {None}
	}
	fn mapCode(code: RefCode<'a>) -> Option<&'a Self> {
		if let RefCode::Skeleton(s) = code {Some(s as &Self)}
		else {None}
	}
	fn getData(&self, data: &mut Self::Data, ctx: &GameContext) {
		let player = ctx.getHolder().getTyped(ctx.getPlayerID()).unwrap();
		let playerDirection = Vector::fromPoints(player.getPosition(), self.position);
		data.nextPos = self.position + playerDirection.normalize() * 3f32;
	}
	fn update(&mut self, data: &Self::Data, _po: &PO) {
		self.position = data.nextPos;
		self.updatePositions();

		if !self.idle {
			self.timer += 1;
			if self.timer == 30 {
				self.animationsBottom.changeAnimation(ANIMATION_IDX_BOTTOM::Walk1 as usize);
			}
			if self.timer > 60 {
				self.timer = 1;
				self.animationsBottom.changeAnimation(ANIMATION_IDX_BOTTOM::Walk0 as usize);
			}
		}
			
	}
	fn needsExecution(&self) -> bool {true}
	fn tick(&mut self) {}
	fn draw(&self, canvas: &mut Canvas<Window>) {
		self.animationsTop.drawNextFrame(canvas, self.renderPositionTop);
		self.animationsBottom.drawNextFrame(canvas, self.renderPositionBottom);
	}
}

