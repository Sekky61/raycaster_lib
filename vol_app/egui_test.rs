use std::convert::TryInto;

use pushrod::render::engine::Engine;
use pushrod::render::widget::{BaseWidget, Widget};
use pushrod::render::{make_points, make_size, Points, Size};
use pushrod::widgets::checkbox_widget::CheckboxWidget;
use pushrod::widgets::text_widget::TextWidget;
use sdl2::pixels::Color;

use pushrod::widgets::text_widget::TextJustify;
use sdl2::ttf::FontStyle;

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("pushrod", 600, 400)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut engine = Engine::new(600, 400, 30);

    let config_id = 45;

    let mut base_widget = BaseWidget::new(vec![20, 20], vec![560, 360]);
    base_widget.set_color(config_id, Color::RGBA(127, 127, 127, 255));
    let base_widget_id = engine.add_widget(Box::new(base_widget), "sys_wid_name".into());

    eprintln!("Added base widget ID: {}", base_widget_id);

    let font_path = std::env::current_dir().expect("No curr dir");
    let font_path = font_path.join("Roboto-Regular.ttf");
    let font_path = font_path.to_str().expect("err on path").to_owned();

    let mut text_widget = TextWidget::new(
        "assets/Roboto-Regular.ttf".into(),
        FontStyle::NORMAL,
        96,
        TextJustify::Left,
        String::from("Hello, Pushrod World!"),
        vec![0, 20],
        vec![600, 40],
    );
    let text_widget_id1 = engine.add_widget(Box::new(text_widget), "sys_wid_name2".into());

    let box1_config_id = 77;

    let mut box_widget1 =
        CheckboxWidget::new(vec![40, 40], vec![400, 100], "box1_name".into(), 40, false);
    box_widget1.set_color(box1_config_id, Color::RGB(200, 0, 0));
    let box_widget_id1 = engine.add_widget(Box::new(box_widget1), "box1_name2".into());

    eprintln!("Added box widget ID: {}", box_widget_id1);

    let box2_config_id = 99;

    let mut box_widget2 =
        CheckboxWidget::new(vec![40, 180], vec![400, 100], "box2_name".into(), 40, false);
    box_widget2.set_color(box2_config_id, Color::RGB(20, 200, 0));
    let box_widget_id2 = engine.add_widget(Box::new(box_widget2), "box2_name2".into());

    eprintln!("Added box widget ID: {}", box_widget_id2);

    let mut new_base_widget = BaseWidget::new(make_points(100, 100), make_size(600, 400));

    let CONFIG_COLOR_BORDER = 98;
    let CONFIG_BORDER_WIDTH = 97;

    new_base_widget
        .get_config()
        .set_color(box2_config_id, Color::RGB(0, 0, 230));
    new_base_widget
        .get_config()
        .set_numeric(CONFIG_BORDER_WIDTH, 2);
    //
    // new_base_widget
    //     .get_callbacks()
    //     .on_mouse_entered(|x, _widgets, _layouts| {
    //         x.get_config()
    //             .set_color(CONFIG_COLOR_BASE, Color::RGB(255, 0, 0));
    //         x.get_config().set_invalidated(true);
    //         _widgets[0]
    //             .widget
    //             .borrow_mut()
    //             .get_config()
    //             .set_invalidated(true);
    //         eprintln!("Mouse Entered");
    //     });
    //
    // new_base_widget
    //     .get_callbacks()
    //     .on_mouse_exited(|x, _widgets, _layouts| {
    //         x.get_config()
    //             .set_color(CONFIG_COLOR_BASE, Color::RGB(255, 255, 255));
    //         x.get_config().set_invalidated(true);
    //         _widgets[0]
    //             .widget
    //             .borrow_mut()
    //             .get_config()
    //             .set_invalidated(true);
    //         eprintln!("Mouse Exited");
    //     });
    //
    // new_base_widget
    //     .get_callbacks()
    //     .on_mouse_moved(|_widget, _widgets, _layouts, points| {
    //         eprintln!("Mouse Moved: {:?}", points);
    //     });
    //
    // new_base_widget
    //     .get_callbacks()
    //     .on_mouse_scrolled(|_widget, _widgets, _layouts, points| {
    //         eprintln!("Mouse Scrolled: {:?}", points);
    //     });
    //
    // new_base_widget.get_callbacks().on_mouse_clicked(
    //     |_widget, _widgets, _layouts, button, clicks, state| {
    //         eprintln!(
    //             "Mouse Clicked: button={} clicks={} state={}",
    //             button, clicks, state
    //         );
    //     },
    // );
    //
    // engine.add_widget(Box::new(new_base_widget), String::from("widget1"));
    //
    // engine.on_exit(|engine| {
    //     let buttons: Vec<_> = vec![
    //         ButtonData {
    //             flags: MessageBoxButtonFlag::RETURNKEY_DEFAULT,
    //             button_id: 1,
    //             text: "Yes",
    //         },
    //         ButtonData {
    //             flags: MessageBoxButtonFlag::ESCAPEKEY_DEFAULT,
    //             button_id: 2,
    //             text: "No",
    //         },
    //     ];
    //
    //     let res = show_message_box(
    //         MessageBoxFlag::WARNING,
    //         buttons.as_slice(),
    //         "Quit",
    //         "Are you sure?",
    //         None,
    //         None,
    //     )
    //         .unwrap();
    //
    //     if let ClickedButton::CustomButton(x) = res {
    //         if x.button_id == 1 {
    //             return true;
    //         }
    //     }
    //
    //     false
    // });

    engine.run(sdl_context, window);
}
