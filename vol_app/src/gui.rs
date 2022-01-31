use std::time::Duration;

use nalgebra::{Point3, Vector3};
use pushrod::{
    base_widget::BaseWidget,
    box_widget::BoxWidget,
    cache::WidgetCache,
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
const BG_COLOR: Color = Color::RGB(20, 20, 20);
const LEFT_MENU_COLOR: Color = Color::RGB(50, 50, 50);

pub struct State {
    cam_coords: Point3<f32>,
}

impl State {
    pub fn new() -> Self {
        Self {
            cam_coords: Point3::origin(),
        }
    }
}

pub struct Gui {
    pub cache: WidgetCache,
    pub window_size: (u32, u32),
    pub base_widget_id: i32,
    pub left_menu_id: i32,
    pub ms_counter_title_id: i32,
    pub ms_counter_id: i32,
    pub cam_pos_title_id: i32,
    pub cam_pos_id: i32,
    pub sphere_pos_id: i32,
    pub state: State,
}

impl Gui {
    pub fn new() -> Gui {
        Gui {
            cache: WidgetCache::default(),
            window_size: (WIN_W, WIN_H),
            base_widget_id: -1,
            left_menu_id: -1,
            ms_counter_title_id: -1,
            ms_counter_id: -1,
            cam_pos_title_id: -1,
            cam_pos_id: -1,
            state: State::new(),
            sphere_pos_id: -1,
        }
    }

    pub fn send_cam_pos(&mut self, pos: Point3<f32>) {
        if self.state.cam_coords == pos {
            return;
        }

        if let Some(SystemWidget::Text(cam_pos_widget)) = self.cache.get_mut(self.cam_pos_id) {
            let coord_text = format!("[ {:>6.1} , {:>6.1} , {:>6.1} ]", pos.x, pos.y, pos.z);
            cam_pos_widget.set_text(coord_text.as_str());
        }

        self.state.cam_coords = pos;
    }

    pub fn send_frame_time(&mut self, time: Duration) {
        if let Some(SystemWidget::Text(ms_counter)) = self.cache.get_mut(self.ms_counter_id) {
            let ms_text = time.as_millis().to_string();
            ms_counter.set_text(ms_text.as_str());
        }
    }

    pub fn send_spherical_pos(&mut self, coords: (f32, f32, f32)) {
        if let Some(SystemWidget::Text(sphere_pos_widget)) = self.cache.get_mut(self.sphere_pos_id)
        {
            let coord_text = format!(
                "[ {:>6.1} , {:>6.1} , {:>6.1} ]",
                coords.0, coords.1, coords.2
            );
            sphere_pos_widget.set_text(coord_text.as_str());
        }
    }

    pub fn handle_event(&mut self, event: Event) -> Vec<PushrodEvent> {
        self.cache.handle_event(event)
    }

    pub fn build_gui(&mut self) {
        // Background
        let mut base_widget = BaseWidget::new(
            Point::new(0, 0),
            Size::new(self.window_size.0, self.window_size.1),
        );
        base_widget.set_color(BG_COLOR);
        self.base_widget_id = self.cache.add(SystemWidget::Base(Box::new(base_widget)));

        // Left menu
        let mut box_widget1 = BoxWidget::new(
            Point::new(DEFAULT_PADDING, DEFAULT_PADDING),
            LEFT_MENU_SIZE,
            Color::YELLOW,
            1,
        );
        box_widget1.set_color(LEFT_MENU_COLOR);
        self.left_menu_id = self.cache.add(SystemWidget::Box(Box::new(box_widget1)));

        // ms counter title
        let mut ms_counter_title = TextWidget::new(
            Point::new(2 * DEFAULT_PADDING, 2 * DEFAULT_PADDING),
            Size::new(LEFT_MENU_SIZE.w - 2 * (DEFAULT_PADDING as u32), 30),
            "Frame time".into(),
            TextAlignment::AlignLeft,
        );
        ms_counter_title.set_bg_color(LEFT_MENU_COLOR);
        self.ms_counter_title_id = self
            .cache
            .add(SystemWidget::Text(Box::new(ms_counter_title)));

        // ms counter
        let mut ms_counter = TextWidget::new(
            Point::new(2 * DEFAULT_PADDING, 3 * DEFAULT_PADDING + 30),
            Size::new(LEFT_MENU_SIZE.w - 2 * (DEFAULT_PADDING as u32), 30),
            "def ms".into(),
            TextAlignment::AlignLeft,
        );
        ms_counter.set_bg_color(LEFT_MENU_COLOR);
        self.ms_counter_id = self.cache.add(SystemWidget::Text(Box::new(ms_counter)));

        // camera position title
        let mut cam_pos_title = TextWidget::new(
            Point::new(2 * DEFAULT_PADDING, 4 * DEFAULT_PADDING + 2 * 30),
            Size::new(LEFT_MENU_SIZE.w - 2 * (DEFAULT_PADDING as u32), 30),
            "Camera position".into(),
            TextAlignment::AlignLeft,
        );
        cam_pos_title.set_bg_color(LEFT_MENU_COLOR);
        self.cam_pos_title_id = self.cache.add(SystemWidget::Text(Box::new(cam_pos_title)));

        // camera position
        let mut cam_pos = TextWidget::new(
            Point::new(2 * DEFAULT_PADDING, 5 * DEFAULT_PADDING + 3 * 30),
            Size::new(LEFT_MENU_SIZE.w - 2 * (DEFAULT_PADDING as u32), 30),
            "deez".into(),
            TextAlignment::AlignLeft,
        );
        cam_pos.set_bg_color(LEFT_MENU_COLOR);
        self.cam_pos_id = self.cache.add(SystemWidget::Text(Box::new(cam_pos)));

        // camera position spherical
        let mut sphere_pos = TextWidget::new(
            Point::new(2 * DEFAULT_PADDING, 6 * DEFAULT_PADDING + 4 * 30),
            Size::new(LEFT_MENU_SIZE.w - 2 * (DEFAULT_PADDING as u32), 30),
            "kuuma".into(),
            TextAlignment::AlignLeft,
        );
        sphere_pos.set_bg_color(LEFT_MENU_COLOR);
        self.sphere_pos_id = self.cache.add(SystemWidget::Text(Box::new(sphere_pos)));
    }
}
