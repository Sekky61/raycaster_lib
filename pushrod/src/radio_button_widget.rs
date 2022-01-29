use sdl2::image::LoadTexture;
use sdl2::render::{Canvas, Texture};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::Window;

use crate::event::PushrodEvent;
use crate::geometry::{Point, Size};
use crate::text_widget::TextWidget;
use crate::texture::TextureStore;
use crate::widget::Widget;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::any::Any;
use std::cmp::min;

/// Internal flag indicating if the button is currently accepting mouse button focus.
const PROPERTY_BUTTON_ACTIVE: u32 = 20000;

/// Internal flag indicating if the mouse is in the bounds of the `Widget`.
const PROPERTY_BUTTON_IN_BOUNDS: u32 = 20001;

/// Internal flag indicating whether or not this `Widget` originated the `MouseButton` state of `true`.
const PROPERTY_BUTTON_ORIGINATED: u32 = 20002;

/// Internal flag indicating whether or not this `Widget` has the mouse currently hovered over it, and
/// the button state is highlighted.
const PROPERTY_BUTTON_HOVERED: u32 = 20003;

/// Base Widget.
pub struct RadioButtonWidget {
    origin: Point,
    size: Size,
    texture: TextureStore,
    hovered: bool,
    toggled: bool,
    invalidated: bool,
    group_id: u32,
    widget_id: u32,
    text_widget: TextWidget,
}

/// Auxiliary functions for the `RadioButtonWidget`.
impl RadioButtonWidget {
    /// Sets the state to hovered, meaning the mouse has entered the bounds of the `Widget`.
    fn set_hovered(&mut self) {
        self.hovered = true;
        self.set_invalidated(true);
    }

    /// Sets the state to unhovered, meaning the mouse has left the bounds of the `Widget`.
    fn set_unhovered(&mut self) {
        self.hovered = false;
        self.set_invalidated(true);
    }

    /// Swaps the toggled state.
    fn toggle_selected(&mut self) {
        self.toggled = !self.toggled;
    }

    /// Shortcut function to determine if the button is currently in a selected state or not.
    fn is_selected(&self) -> bool {
        self.toggled
    }

    /// Shortcut function to determine if the button is currently being hovered over.
    fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn radio_draw(
        &mut self,
        c: &mut Canvas<Window>,
        ttf_context: &Sdl2TtfContext,
    ) -> Option<&Texture> {
        // ONLY update the texture if the `BaseWidget` shows that it's been invalidated.
        if self.is_invalidated() {
            let text_color = Some(Color::BLACK);
            let (font_texture, width, height) = self.text_widget.render_text(c, ttf_context);
            let widget_w = self.size.w;

            self.texture.create_or_resize_texture(c, self.size);

            let display_image = if self.is_selected() {
                if self.is_hovered() {
                    String::from("assets/radio_unselected.png")
                } else {
                    String::from("assets/radio_selected.png")
                }
            } else if self.is_hovered() {
                String::from("assets/radio_selected.png")
            } else {
                String::from("assets/radio_unselected.png")
            };

            let border_width = 3;
            let LEFT = 89;
            let text_justification = LEFT;

            let checkbox_texture = c
                .texture_creator()
                .load_texture(display_image)
                .expect("Cant load texture");

            let image_size = min(self.size.h as u32, 32);

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                //draw_base(texture, &cloned_properties, Some(back_color));

                let start_font_y = if height > self.size.h {
                    border_width as u32
                } else {
                    (self.size.h / 2) - (height / 2) - (border_width / 2) as u32
                };

                let start_font_x = if text_justification == LEFT {
                    border_width + image_size as i32 + 6 // 6 pixels of padding
                } else {
                    (self.size.w - width - image_size - 6) as i32 // 6 pixels of padding
                };

                let checkbox_start_x = if text_justification == LEFT {
                    border_width as u32
                } else {
                    self.size.w - (border_width * 2) as u32 - image_size
                };

                texture
                    .copy(
                        &font_texture,
                        None,
                        Rect::new(start_font_x, start_font_y as i32, width, height),
                    )
                    .unwrap();

                texture
                    .copy(
                        &checkbox_texture,
                        None,
                        Rect::new(
                            checkbox_start_x as i32,
                            (self.size.h / 2 - image_size / 2) as i32,
                            image_size,
                            image_size,
                        ),
                    )
                    .unwrap();
            })
            .unwrap();
        }

        self.texture.get_optional_ref()
    }
}

impl Widget for RadioButtonWidget {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_origin(&self) -> &Point {
        &self.origin
    }

    fn get_size(&self) -> &Size {
        &self.size
    }

    fn get_color(&self) -> Color {
        Color::WHITE
    }

    fn set_origin(&mut self, point: Point) {
        self.origin = point;
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.set_invalidated(true);
    }

    fn set_invalidated(&mut self, state: bool) {
        self.invalidated = state;
    }

    fn set_color(&mut self, color: Color) {
        // noop
        self.set_invalidated(true);
    }

    fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    fn get_texture(&mut self) -> &mut TextureStore {
        &mut self.texture
    }

    fn handle_event(&mut self, event: PushrodEvent) -> Option<Vec<PushrodEvent>> {
        match event {
            PushrodEvent::EnteredBounds(x) => {
                self.set_hovered();
            }
            PushrodEvent::ExitedBounds(x) => {
                self.set_unhovered();
            }
            PushrodEvent::WidgetRadioSelected(widget_id, group_id) => {
                if self.group_id == group_id {
                    if self.widget_id == widget_id {
                        self.toggled = true;
                    } else {
                        self.toggled = false;
                    }
                }
            }
            PushrodEvent::Clicked(widget_id, clicked_times) => {
                if !self.toggled {
                    self.toggled = true;

                    let ev = PushrodEvent::WidgetRadioSelected(self.widget_id, self.group_id);

                    return Some(vec![ev]);
                }
            }
            _ => eprintln!("[RadioWidget::handle_event] unhandled event {:?}", event),
        }

        None
    }

    fn draw(&mut self, c: &mut Canvas<Window>) -> Option<&Texture> {
        panic!("Use radio_draw!!!");
    }
}
