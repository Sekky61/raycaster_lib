use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};

use raycaster_lib::{
    volumetric::{BlockVolume, LinearVolume},
    RenderOptions, Renderer,
};

fn main() {
    let app = Application::builder()
        .application_id("org.example.HelloWorld")
        .build();

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

    let mut buffer = vec![0; 3 * 512 * 512];

    raycast_renderer.render_to_buffer(buffer.as_mut_slice());

    let buf = raycast_renderer.get_buffer();

    app.connect_activate(move |app| {
        let bytes = gtk::glib::Bytes::from_owned(buf.clone());
        let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_bytes(
            &bytes,
            gtk::gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            512,
            512,
            512 * 3,
        );

        let imgb = gtk::ImageBuilder::new();
        let img = imgb.pixbuf(&pixbuf).build();
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
        grid.add(&img);
        win.add(&grid);

        // Don't forget to make all widgets visible.
        win.show_all();
    });

    app.run();
}
