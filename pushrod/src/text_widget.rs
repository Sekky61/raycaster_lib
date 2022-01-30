// Text Widget
// Pushrod
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::event::PushrodEvent;
use crate::geometry::{Point, Size};
use crate::texture::TextureStore;
use crate::widget::Widget;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureQuery};
use sdl2::ttf::{FontStyle, Sdl2TtfContext};
use sdl2::video::Window;
use std::any::Any;
use std::path::Path;

/// `TextAlignment` is used by the `TextWidget` to identify the alignment of the text within the
/// bounds of the `Widget`.
pub enum TextAlignment {
    /// Align text against the left bounds.
    AlignLeft,

    /// Align text to the center of the bounds.
    AlignCenter,

    /// Align text so the end of the text is against the max width of the bounds.
    AlignRight,
}

pub struct TextWidget {
    origin: Point,
    size: Size,
    invalidated: bool,
    texture: TextureStore,
    text: String,
    alignment: TextAlignment,
    font_name: String,
    font_size: u16,
    font_color: Color,
    bg_color: Color,
}

/// `TextWidget` is a widget that renders text from a string within the bounds of the `Widget`.
impl Widget for TextWidget {
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
        Color::BLACK
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

    fn set_color(&mut self, _color: Color) {}

    fn is_invalidated(&self) -> bool {
        self.invalidated
    }

    fn get_texture(&mut self) -> &mut TextureStore {
        &mut self.texture
    }

    fn handle_event(&mut self, event: PushrodEvent) -> Option<Vec<PushrodEvent>> {
        match event {
            PushrodEvent::SystemEvent(ev) => {
                eprintln!("[TextWidget::handle_event] event: {:?}", ev);
            }

            _ => {}
        }

        None
    }

    fn draw(&mut self, c: &mut Canvas<Window>) -> Option<&Texture> {
        panic!("Use text_draw!!!")
    }
}

impl TextWidget {
    pub fn new(origin: Point, size: Size, text: String, align: TextAlignment) -> Self {
        Self {
            origin,
            size,
            invalidated: false,
            texture: TextureStore::default(),
            text,
            alignment: align,
            font_name: "pushrod/assets/OpenSans-Regular.ttf".into(),
            font_size: 16,
            font_color: Color::WHITE,
            bg_color: Color::BLACK,
        }
    }

    pub fn set_bg_color(&mut self, color: Color) {
        self.bg_color = color;
        self.invalidated = true;
    }

    pub fn set_font_color(&mut self, color: Color) {
        self.font_color = color;
        self.invalidated = true;
    }

    pub fn set_font_size(&mut self, size: impl Into<u16>) {
        self.font_size = size.into();
        self.invalidated = true;
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_owned();
        self.invalidated = true;
    }

    pub fn get_text(&self) -> &str {
        self.text.as_str()
    }

    pub fn text_draw(
        &mut self,
        c: &mut Canvas<Window>,
        ttf_context: &Sdl2TtfContext,
    ) -> Option<&Texture> {
        if self.invalidated {
            let (font_texture, width, height) = self.render_text(c, ttf_context);
            let widget_w = self.size.w;
            let texture_x: i32 = match self.alignment {
                TextAlignment::AlignLeft => 0,
                TextAlignment::AlignCenter => (widget_w - width) as i32 / 2,
                TextAlignment::AlignRight => (widget_w - width) as i32,
            };

            self.texture.create_or_resize_texture(c, self.size);

            c.with_texture_canvas(self.texture.get_mut_ref(), |texture| {
                texture.set_draw_color(self.bg_color);
                texture.clear(); // Without clear, only text gets drawn

                texture
                    .copy(&font_texture, None, Rect::new(texture_x, 0, width, height))
                    .unwrap();
            })
            .unwrap();
        }

        self.texture.get_optional_ref()
    }

    /// Renders text, given the font name, size, style, color, string, and max width.  Transfers
    /// ownership of the `Texture` to the calling function, returns the width and height of the
    /// texture after rendering.  By using the identical font name, size, and style, if SDL2 caches
    /// the font data, this will allow the font to be cached internally.
    pub fn render_text(
        &mut self,
        c: &mut Canvas<Window>,
        ttf_context: &Sdl2TtfContext,
    ) -> (Texture, u32, u32) {
        let texture_creator = c.texture_creator();
        let font_style: FontStyle = FontStyle::NORMAL;
        let text_message = self.text.as_str();

        let mut font = ttf_context
            .load_font(Path::new(self.font_name.as_str()), self.font_size)
            .unwrap();
        let surface = font
            .render(text_message)
            .blended_wrapped(self.font_color, self.size.w)
            .map_err(|e| e.to_string())
            .unwrap();

        let font_texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())
            .unwrap();
        let TextureQuery { width, height, .. } = font_texture.query();

        font.set_style(font_style);

        (font_texture, width, height)
    }
}
