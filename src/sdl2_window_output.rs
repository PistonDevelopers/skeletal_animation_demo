use std::rc::Rc;
use std::cell::RefCell;

use gfx::traits::*;
use gfx;
use piston;
use piston::window::{Window, Size};
use sdl2_window::Sdl2Window;

// TEMP -- Need wrapper for Sdl2Window that implements gfx::Output
pub struct WindowOutput<R: gfx::Resources> {
    pub window: Rc<RefCell<Sdl2Window>>,
    frame: gfx::FrameBufferHandle<R>,
    mask: gfx::Mask,
    gamma: gfx::Gamma,
}

impl<R: gfx::Resources> gfx::Output<R> for WindowOutput<R> {

    fn get_handle(&self) -> Option<&gfx::FrameBufferHandle<R>> {
        Some(&self.frame)
    }

    fn get_size(&self) -> (gfx::tex::Size, gfx::tex::Size) {
        let Size {width: w, height: h} = self.window.borrow().size();
        (w as gfx::tex::Size, h as gfx::tex::Size)
    }

    fn get_mask(&self) -> gfx::Mask {
        self.mask
    }

    fn get_gamma(&self) -> gfx::Gamma {
        self.gamma
    }
}

impl<R: gfx::Resources> WindowOutput<R> {
    pub fn new(window: Rc<RefCell<Sdl2Window>>, frame: gfx::FrameBufferHandle<R>) -> WindowOutput<R> {
        WindowOutput {
            window: window.clone(),
            frame: frame,
            mask: gfx::COLOR | gfx::DEPTH | gfx::STENCIL,
            gamma: gfx::Gamma::Original
        }
    }
}
