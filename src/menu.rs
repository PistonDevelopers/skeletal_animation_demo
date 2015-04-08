use gfx_debug_draw::DebugRenderer;
use gfx_device_gl::Resources as GlResources; // FIXME

use piston::event::{GenericEvent, PressEvent, ReleaseEvent};
use piston::input::keyboard::Key;
use piston::input::Button::Keyboard;

type ItemIndex = usize;

pub struct Settings {
    pub draw_skeleton: bool,
    pub draw_labels: bool,
    pub draw_mesh: bool,
    pub playback_speed: f32,
}

pub struct Menu {
    items: Vec<MenuItem>,
    selected_item: ItemIndex,
}

impl Menu {

    pub fn new() -> Menu {
        Menu {
            items: Vec::new(),
            selected_item: 0,
        }
    }

    pub fn add_item(&mut self, text: &str, action: Box<Fn(&mut Settings) -> ()>) {
        self.items.push(MenuItem{
            text: text.to_string(),
            action: action,
        });
    }

    ///
    /// FIXME - make backend-agnostic
    ///
    pub fn draw(&self, debug_renderer: &mut DebugRenderer<GlResources>) {

        let left_margin = 10;
        let top_margin = 10;
        let item_space = 20;

        for (index, item) in self.items.iter().enumerate() {
            let selected = index == self.selected_item;
            item.draw(debug_renderer, [left_margin, top_margin + item_space * index as i32], selected);
        }
    }

    ///
    /// Respond to keyboard control events
    ///
    pub fn event<E: GenericEvent>(&mut self, e: &E, settings: &mut Settings) {

        e.press(|button| {
            match button {

                Keyboard(Key::Up) => {
                    self.selected_item = self.selected_item.wrapping_sub(1) % self.items.len();
                }

                Keyboard(Key::Down) => {
                    self.selected_item = self.selected_item.wrapping_add(1) % self.items.len();
                }

                Keyboard(Key::Space) => {
                    let item = self.get_selected_item();
                    (*item.action)(settings);
                }

                _ => {}
            }
        });

        /*
        e.release(|button| {
            match button {
                x if x == self.settings.orbit_button => self.keys.remove(ORBIT),
                x if x == self.settings.pan_button => self.keys.remove(PAN),
                x if x == self.settings.zoom_button => self.keys.remove(ZOOM),
                _ => {}
            }
        });
        */
    }

    pub fn get_selected_item(&self) -> &MenuItem {
        &self.items[self.selected_item]
    }
}

pub struct MenuItem {
    text: String,
    action: Box<Fn(&mut Settings) -> ()>,
}

impl MenuItem {
    pub fn draw(&self, debug_renderer: &mut DebugRenderer<GlResources>, position: [i32; 2], selected: bool) {

        let color = if selected {
            [1.0, 0.5, 0.5, 1.0]
        } else {
            [0.5, 0.5, 0.5, 1.0]
        };

        debug_renderer.draw_text_on_screen(
            &self.text[..],
            position,
            color,
        );
    }
}
