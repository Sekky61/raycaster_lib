use pushrod::{
    base_widget::BaseWidget,
    box_widget::BoxWidget,
    engine::Engine,
    event::PushrodEvent,
    geometry::{Point, Size},
    text_widget::{TextAlignment, TextWidget},
    widget::{SystemWidget, Widget},
};
use sdl2::{event::Event, pixels::Color};

pub const WIN_W: u32 = 980;
pub const WIN_H: u32 = 720;

const DEFAULT_PADDING: i32 = 10;
const LEFT_MENU_SIZE: Size = Size::new(250, 700);
const LEFT_MENU_COLOR: Color = Color::RGB(50, 50, 50);

pub struct Gui {
    pub engine: Engine,
    pub window_size: (u32, u32),
    pub base_widget_id: i32,
    pub left_menu_id: i32,
    pub ms_counter_id: i32,
}

impl Gui {
    pub fn new() -> Gui {
        // frame_rate has no effect now
        let engine = Engine::new(Size::new(WIN_W, WIN_H), 60);
        Gui {
            engine,
            window_size: (WIN_W, WIN_H),
            base_widget_id: -1,
            left_menu_id: -1,
            ms_counter_id: -1,
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Vec<PushrodEvent> {
        self.engine.widget_cache.handle_event(event)
    }

    pub fn build_gui(&mut self) {
        // Background
        let mut base_widget = BaseWidget::new(
            Point::new(0, 0),
            Size::new(self.window_size.0, self.window_size.1),
        );
        base_widget.set_color(Color::RGBA(20, 20, 20, 255));
        self.base_widget_id = self
            .engine
            .add_widget(SystemWidget::Base(Box::new(base_widget)));

        // Left menu
        let mut box_widget1 = BoxWidget::new(
            Point::new(DEFAULT_PADDING, DEFAULT_PADDING),
            LEFT_MENU_SIZE,
            Color::YELLOW,
            1,
        );
        box_widget1.set_color(LEFT_MENU_COLOR);
        self.left_menu_id = self
            .engine
            .add_widget(SystemWidget::Box(Box::new(box_widget1)));

        // ms counter
        let mut ms_counter = TextWidget::new(
            Point::new(20, 20),
            Size::new(200, 50),
            "def ms".into(),
            TextAlignment::AlignLeft,
        );
        ms_counter.set_invalidated(true);
        self.ms_counter_id = self
            .engine
            .add_widget(SystemWidget::Text(Box::new(ms_counter)));
    }
}
