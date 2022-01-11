use std::sync::Arc;
use std::thread;

use gtk::gdk_pixbuf::Colorspace;
use gtk::{glib, prelude::*};
use gtk::{Application, ApplicationWindow};

use nalgebra::vector;
use parking_lot::RwLock;
use raycaster_lib::render::BufferStatus;
use raycaster_lib::{
    volumetric::{BlockVolume, LinearVolume},
    RenderOptions, Renderer,
};

fn main() {
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();

    let frame1 = Arc::new(RwLock::new(vec![0; 3 * 512 * 512]));
    let frame1_cpy = frame1.clone();

    let frame2 = Arc::new(RwLock::new(vec![0; 3 * 512 * 512]));
    let frame2_cpy = frame2.clone();

    let bufferstatus = Arc::new(RwLock::new(BufferStatus::new()));
    let bufferstatus_checker = bufferstatus.clone();
    let bufferstatus_cpy = bufferstatus.clone();

    app.connect_activate(move |app| {
        let mut img = gtk::Image::new();

        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

        thread::spawn(move || loop {
            let mut buf = vec![0; 3 * 512 * 512];

            let volume = raycaster_lib::vol_reader::from_file("Skull.vol")
                .expect("bad read of file")
                .build();

            let camera = raycaster_lib::Camera::new(512, 512);

            let mut raycast_renderer = Renderer::<BlockVolume>::new(volume, camera);

            raycast_renderer.set_render_options(RenderOptions {
                ray_termination: true,
                empty_index: true,
                multi_thread: false,
            });

            raycast_renderer.render_to_buffer(&mut buf);

            tx.send(buf);

            raycast_renderer.change_camera_pos(vector![20.0, 20.0, 20.0]);
        });

        rx.attach(None, move |buf| {
            let buf = glib::Bytes::from_owned(buf);
            let pixbuf =
                gtk::gdk_pixbuf::Pixbuf::from_bytes(&buf, Colorspace::Rgb, false, 8, 512, 512, 0);
            img = gtk::Image::from_pixbuf(Some(&pixbuf));
            glib::Continue(true)
        });

        // We create the main window.
        let win = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("Hello, World!")
            .build();

        let label = gtk::Label::new(None);
        label.set_text("geddit");
        // win.add(&label);

        // win.add(&img);
        let grid = gtk::GridBuilder::new().row_spacing(200).build();
        grid.add(&label);
        //grid.add(&img);
        win.add(&grid);

        // Don't forget to make all widgets visible.
        win.show_all();
    });

    app.run();
}
