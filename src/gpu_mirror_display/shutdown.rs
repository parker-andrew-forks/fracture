use winit::event_loop::ActiveEventLoop;

use super::state::{AdditionalRenderingState, State};

pub fn start_shutdown(s: &mut State) {
    s.should_shutdown = true;
}

pub fn shutdown(ev: &ActiveEventLoop, _state: &State, additional: &AdditionalRenderingState) {
    println!("Shutting down.");

    let _ = additional.channels.terminate_pipewire_stream.send(());
    let _ = additional.channels.terminate_settings_ui.send(());

    {
        // The termination check is completed when starting
        additional.open_settings_ui();
        additional.shutdown_settings_ui();
    }

    // wait for signals or channel errors
    let _ = additional.channels.ui_shutdown_conf.recv();
    let _ = additional.channels.dbus_shutdown_conf.recv();

    ev.exit();
}
