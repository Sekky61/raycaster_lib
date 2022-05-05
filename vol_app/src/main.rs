/*
    vol_app
    Author: Michal Majer
    Date: 2022-05-05
*/

//! Volume rendering demo app
//!
//! Launch without arguments, for example:
//! `cargo run --release --bin vol_app`

use crossbeam_channel::{select, Receiver, Sender};

// GUI bindings
slint::include_modules!();

mod app;
use app::State;

pub fn main() {
    // GUI App object and handles
    let app = App::new();
    let app_weak = app.as_weak();
    let app_poll = app_weak.clone();

    // State
    // Wrapped for access in closures
    let mut state = State::new_shared(app_weak);

    state.initial_render_call();

    state.sync_state_with_gui();

    // Start thread listening for messages from renderer
    let (shutdown_send, shutdown_recv): (Sender<_>, Receiver<()>) = crossbeam_channel::bounded(2);
    let render_msg_recv_thread = {
        let state_mut = state.borrow_mut();
        let render_recv = state_mut.get_renderer_receiver();
        std::thread::spawn(move || loop {
            select! {
                recv(shutdown_recv) -> _ => return,
                recv(render_recv) -> msg => {
                    match msg {
                        Ok(()) => (),
                        Err(_) => return
                    }
                }
            }

            let a = app_poll.clone();
            slint::invoke_from_event_loop(move || a.unwrap().invoke_new_rendered_frame());
        })
    };

    //
    // Registering callbacks
    //

    state.register_callbacks(&app);

    // Run app, main event loop
    app.show();
    slint::run_event_loop();
    app.hide();

    // Shutdown
    println!("App shutting down");
    state.shutdown_renderer();

    shutdown_send.send(()).unwrap();
    render_msg_recv_thread.join().unwrap();
}
