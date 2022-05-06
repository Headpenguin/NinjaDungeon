extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::image::LoadTexture;
use sdl2::video::{WindowContext, Window};

use std::ops::Deref;
use std::io::{self, ErrorKind, Error};
use std::fs;

const MIRROR_PATTERN: &'static str = "__half";

mod Parsing;

use Parsing::*;

fn loadSprites<'a, 'b> (creator: &'a TextureCreator<WindowContext>, filenames: &'b [&'b str]) -> Result<Vec<Sprite<'a>>, String> {
    let mut sprites = vec![];
    for filename in filenames {
        sprites.push(Sprite(creator.load_texture(filename)?, filename.contains(MIRROR_PATTERN)));
    }
    Ok(sprites)
}

pub struct Sprites<'a> {
    sprites: Vec<Sprite<'a>>,
}

impl<'a> Sprites<'a> {
    pub fn new<'b> (creator: &'a TextureCreator<WindowContext>, filenames: &'b [&'b str]) -> Result<Sprites<'a>, String> {
        Ok(Sprites {
            sprites: loadSprites(creator, filenames)?,
        })
    }
    
	pub fn getSprite(&self, idx: usize) -> &Sprite {
        &self.sprites[idx]
    }
}

pub struct Animations<'a> {
    animations: Vec<Animation<'a>>,
    activeAnimation: usize,
    frameCounter: usize,
}

impl<'a> Animations<'a> {
    pub fn new(filename: &str, animationPaths: &[&str], creator: &'a TextureCreator <WindowContext>) -> io::Result<Animations<'a>> {
		let file = fs::read_to_string(filename)?;
		let animationPaths = parse(&file, animationPaths)?;

		let mut animations = vec![];
		for animation in animationPaths {
			match animation {
				ParsedAnimation::Flip(p) => animations.push(Animation::Flip(FlipAnimation::new(p))),
				ParsedAnimation::Standard(paths, positions) => animations.push(Animation::Standard(
						match StandardAnimation::fromFiles(creator, &paths, &positions) {
							Ok(animation) => animation,
							Err(m) => return Err(Error::new(ErrorKind::Other, m)),
						}
					)),
			}
		}
        Ok(Animations{animations, activeAnimation: 0, frameCounter: 0,})
    }

    pub fn update(&mut self) {
        self.frameCounter = (self.frameCounter + 1) % usize::max_value();
    }

    fn getAnimation<'b>(&'b self) -> &Animation<'b> {
        &self.animations[self.activeAnimation]
    }

    pub fn drawNextFrame(&self, canvas: &mut Canvas<Window>, position: Rect) {
        match self.getAnimation() {

            Animation::Standard(animation) => 
                animation.getFrame(self.frameCounter)
                .draw(canvas, position, false, false),

            Animation::Flip(animation) => 
                if let Animation::Standard(animation) = &self.animations[animation.getIndex()] {
                    animation.getFrame(self.frameCounter)
                    .draw(canvas, position, true, false);
                }
                else {
                    panic!("Flipped animation is a flip of a flipped animation");
                },

        }//.draw(canvas, position, false, false);
    } 

    pub fn changeAnimation(&mut self, idx: usize) -> Result<(), &'static str> {
        if idx >= self.animations.len() {
            return Err("Out of range");
        }

        self.activeAnimation = idx;

        Ok(())
    }

    /*pub fn next(&mut self) -> &'a Sprite<'a> {
        self.frameCounter = (self.frameCounter + 1) % usize::max_value();
        self.animations[self.activeAnimation].getSprite(self.frameCounter)
    }*/
}

/*
pub struct AnimationIter <'a> {
    animations: Animations<'a>,
    activeAnimation: usize,
    frameCounter: usize,
    //phantom: PhantomData <&'b ()>,
}

impl<'a> AnimationIter<'a> {
    pub fn next<'b>(&'b mut self) -> &Sprite<'b> {
        self.frameCounter = (self.frameCounter + 1) % usize::max_value();
        self.animations.getAnimation(self.activeAnimation).getSprite(self.frameCounter)
    }
}*/

pub enum Animation<'a> {
    Flip(FlipAnimation),
    Standard(StandardAnimation<'a>),
}

pub struct FlipAnimation {
    source: usize,
}

impl FlipAnimation {
    pub fn new(source: usize) -> FlipAnimation {
        FlipAnimation{source,}
    }
    pub fn getIndex(&self) -> usize {
        self.source
    }
}

pub struct StandardAnimation<'a> {
    sprites: Sprites<'a>,
    frames: Vec<usize>,
}

impl<'a> StandardAnimation<'a> {
    /*pub fn fromSprites<'b>(sprites: Sprites<'a>, positions: &'b [usize]) -> Result<Animation<'a>, String> {
        let length = sprites.len();

        let mut frames = vec![];

        for position in positions {
            if length > *position {
                frames.push(*position);
            }
            else {
                return Err("Frame out of sprite bounds".to_string());
            }
        }
        
        Ok(Animation{sprites, frames,})
    }*/
    pub fn fromFiles<'b> (creator: &'a TextureCreator<WindowContext>, filenames: &'b [&'b str], positions: &'b [usize]) -> Result<StandardAnimation<'a>, String> {
        let length = filenames.len();
        
        let mut frames = vec![];

        for position in positions {
            if length > *position {
                frames.push(*position);
            }
            else {
                return Err("Frame out of sprite bounds".to_string());
            }
        }
        Ok(StandardAnimation{sprites: Sprites::new(creator, filenames)?, frames,})
    }

    /*pub fn duration(&self) -> usize {
        self.frames.len()
    }*/

    pub fn getFrame(&self, counter: usize) -> &Sprite {
        self.getSprite(self.frames[counter % self.frames.len()])
    }
    
}

impl<'a> Deref for StandardAnimation<'a> {
    type Target = Sprites<'a>;

    fn deref(&self) -> &Self::Target {
        &self.sprites
    }
}

/*impl<'a> Iterator for Animations<'a> {
    type Item = &'a Sprite<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.frameCounter = (self.frameCounter + 1) % usize::max_value();
        Some(self.animations[self.activeAnimation].getSprite(self.frameCounter))
    }

}*/

pub type Mirror = bool;
pub struct Sprite<'a> (Texture<'a>, Mirror);

impl<'a> Sprite<'a> {
	#[allow(unused_must_use)]
	pub fn draw(&self, canvas: &mut Canvas<Window>, quad: Rect, flipHorizontal: bool, flipVertical: bool) {
        if self.1 {
            let mut quad = quad;
            quad.w = (0.5 * quad.w as f32) as i32;
            canvas.copy_ex(&self.0, None, quad, 0f64, None, true, false); //Render the left half
            quad.x += quad.w;
            canvas.copy(&self.0, None, quad); //Render the right half
        }
        else {
            canvas.copy_ex(&self.0, None, quad, 0f64, None, flipHorizontal, flipVertical);
        }
    }
}

