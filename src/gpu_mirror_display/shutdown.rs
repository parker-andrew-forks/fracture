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
        let r1 = additional.open_settings_ui();
        let r2 = additional.shutdown_settings_ui();

        // There's a case where the termination signal kills the thread before the channels are used. There
        // was an expect in the open and closing, so now I'm checking the errors reported to make sure
        // I didn't make any other mistakes that I didn't expect.
        if r1.is_err() || r2.is_err() {
            let result = (r1, r2);
            println!(
                "The Setting UI is predicted to have shutdown already: {:#?}",
                result
            );
        }
    }

    // wait for signals or channel errors
    let res = additional.channels.ui_shutdown_conf.recv();

    println!("The Settings UI is predicted as shutdown: {:#?}", res);

    let res = additional.channels.dbus_shutdown_conf.recv();

    println!("Pipewire is predicted as shutdown: {:#?}", res);

    ev.exit();
}
