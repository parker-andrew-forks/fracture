use winit::event_loop::ActiveEventLoop;

use super::state::{AdditionalRenderingState, State};

pub fn start_shutdown(s: &mut State) {
    s.should_shutdown = true;
}

pub fn shutdown(ev: &ActiveEventLoop, _state: &State, additional: &AdditionalRenderingState) {
    println!("Shutting down.");

    let _ = additional.channels.terminate_pipewire_stream.send(());
    let _ = additional.channels.terminate_settings_ui.send(());

    // This code is REALLY BAD. It just forces the settings UI to restart in the same way that
    // pressing the settings button on the UI does it. When it restarts, the termination channel
    // can be checked.
    {
        let mut current = additional.settings_state.clone();
        current.open_settings_ui = Some(false);
        additional
            .channels
            .gpu_sender_request
            .send(current)
            .expect("Settings thread stays");
        let _ = additional.channels.start_settings_ui.send(());
    }

    // wait for signals or channel errors
    let _ = additional.channels.ui_shutdown_conf.recv();
    let _ = additional.channels.dbus_shutdown_conf.recv();

    ev.exit();
}
