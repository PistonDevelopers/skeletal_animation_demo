use gfx_debug_draw::DebugRenderer;
use gfx_device_gl::Resources as GlResources; // FIXME

use piston::event::{GenericEvent, PressEvent, ReleaseEvent, UpdateEvent};
use piston::input::keyboard::Key;
use piston::input::Button::Keyboard;

type ItemIndex = usize;

pub struct Menu<T> {
    items: Vec<MenuItem<T>>,
    selected_item: ItemIndex,
}

impl<T> Menu<T> {

    pub fn new() -> Menu<T> {
        Menu {
            items: Vec::new(),
            selected_item: 0,
        }
    }

    pub fn add_item(&mut self, item: MenuItem<T>) {
        self.items.push(item);
    }

    ///
    /// FIXME - make backend-agnostic
    ///
    pub fn draw(&self, settings: &T, debug_renderer: &mut DebugRenderer<GlResources>) {

        let left_margin = 10;
        let top_margin = 10;
        let item_space = 20;

        for (index, item) in self.items.iter().enumerate() {
            let selected = index == self.selected_item;
            item.draw(settings, debug_renderer, [left_margin, top_margin + item_space * index as i32], selected);
        }
    }

    ///
    /// Respond to keyboard control events
    ///
    pub fn event<E: GenericEvent>(&mut self, e: &E, settings: &mut T) {

        e.press(|button| {
            match button {

                Keyboard(Key::Up) => {
                    self.selected_item = self.selected_item.wrapping_sub(1) % self.items.len();
                }

                Keyboard(Key::Down) => {
                    self.selected_item = self.selected_item.wrapping_add(1) % self.items.len();
                }

                _ => {}
            }
        });

        self.get_selected_item_mut().event(e, settings);

    }

    pub fn get_selected_item_mut(&mut self) -> &mut MenuItem<T> {
        &mut self.items[self.selected_item]
    }
}

pub enum MenuItem<T> {
    ActionItem(ActionMenuItem<T>),
    SliderItem(SliderMenuItem<T>),
}

impl<T> MenuItem<T> {
    pub fn draw(&self, settings: &T, debug_renderer: &mut DebugRenderer<GlResources>, position: [i32; 2], selected: bool) {
        match self {
            &MenuItem::ActionItem(ref item) => item.draw(settings, debug_renderer, position, selected),
            &MenuItem::SliderItem(ref item) => item.draw(settings, debug_renderer, position, selected),
        }
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, settings: &mut T) {
        match self {
            &mut MenuItem::ActionItem(ref mut item) => item.event(e, settings),
            &mut MenuItem::SliderItem(ref mut item) => item.event(e, settings),
        }
    }

    pub fn action_item(text: &str, action: Box<Fn(&mut T) -> ()>) -> MenuItem<T> {
        MenuItem::ActionItem(ActionMenuItem {
            text: text.to_string(),
            action: action,
        })
    }

    pub fn slider_item(label: &str, range: [f32; 2], step_size: f32, value_getter: Box<Fn(&T) -> f32>, value_setter: Box<Fn(&mut T, f32) -> ()>) -> MenuItem<T> {
        MenuItem::SliderItem(SliderMenuItem {
            label: label.to_string(),
            range: range,
            step_size: step_size,
            get_value: value_getter,
            set_value: value_setter,
            state: SliderMenuState::Default,
        })
    }
}


pub struct ActionMenuItem<T> {
    text: String,
    action: Box<Fn(&mut T) -> ()>,
}

impl<T> ActionMenuItem<T> {
    pub fn draw(&self, _settings: &T, debug_renderer: &mut DebugRenderer<GlResources>, position: [i32; 2], selected: bool) {

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

    pub fn event<E: GenericEvent>(&mut self, e: &E, settings: &mut T) {
        e.press(|button| {
            match button {
                Keyboard(Key::Space) | Keyboard(Key::Left) | Keyboard(Key::Right) => { (*self.action)(settings); },
                _ => {}
            }
        });
    }
}

enum SliderMenuState {
    Default,
    Increasing,
    Decreasing,
}

pub struct SliderMenuItem<T> {
    label: String,
    range: [f32; 2],
    get_value: Box<Fn(&T) -> f32>,
    set_value: Box<Fn(&mut T, f32) -> ()>,
    step_size: f32,
    state: SliderMenuState,
}

impl<T> SliderMenuItem<T> {

    pub fn draw(&self, settings: &T, debug_renderer: &mut DebugRenderer<GlResources>, position: [i32; 2], selected: bool) {

        let color = if selected {
            [1.0, 0.5, 0.5, 1.0]
        } else {
            [0.5, 0.5, 0.5, 1.0]
        };

        let value = (*self.get_value)(settings);

        let text = format!("{} {}", self.label, value);

        debug_renderer.draw_text_on_screen(
            &text[..],
            position,
            color,
        );
    }

    pub fn event<E: GenericEvent>(&mut self, e: &E, settings: &mut T) {


        e.update(|_| {
            match self.state {
                SliderMenuState::Increasing => {
                    let current_value = (*self.get_value)(settings);
                    let new_value = self.range[1].min(current_value + self.step_size);
                    (*self.set_value)(settings, new_value);
                },
                SliderMenuState::Decreasing => {
                    let current_value = (*self.get_value)(settings);
                    let new_value = self.range[0].max(current_value - self.step_size);
                    (*self.set_value)(settings, new_value);
                },
                _ => {}
            }
        });


        e.press(|button| {
            match button {
                Keyboard(Key::Right) => {
                    self.state = SliderMenuState::Increasing
                },
                Keyboard(Key::Left) => {
                    self.state = SliderMenuState::Decreasing
                },
                _ => {}
            }
        });

        e.release(|button| {
            match button {
                Keyboard(Key::Right) | Keyboard(Key::Left) => {
                    self.state = SliderMenuState::Default
                },
                _ => {}
            }
        });
    }

}
