use std::time::Duration;

use winit::event_loop::ActiveEventLoop;

use crate::global_application_state::{GTK_SHUTDOWN_AT_END, PIPEWIRE_SHUTDOWN_AT_END};

use super::state::{AdditionalRenderingState, State};

pub fn start_shutdown(s: &mut State) {
    s.should_shutdown = true;
}

#[derive(Debug)]
pub struct ShutdownResult {
    gtk_settings_ui: Result<(), SettingsGtkShutdownErr>,
    pipewire_threads: Result<(), PipewireShutdownErr>,
}

#[derive(Debug)]
pub enum SettingsGtkShutdownErr {
    ThreadAlreadyTerminated,
    Disconnected(std::sync::mpsc::RecvTimeoutError),
    FailedWithinTimeLimit,
}

#[derive(Debug)]
pub enum PipewireShutdownErr {
    ThreadAlreadyTerminated,
    TimeoutOrTermination(std::sync::mpsc::RecvTimeoutError),
}

pub fn shutdown(
    ev: &ActiveEventLoop,
    _state: &State,
    additional: &AdditionalRenderingState,
) -> Result<ShutdownResult, ShutdownResult> {
    println!("Shutting down.");

    let pw_1 = additional.channels.terminate_pipewire_stream.send(());
    let gtk_1 = additional.channels.terminate_settings_ui.send(());

    let gtk;

    let mut count = 0;

    if gtk_1.is_ok() {
        'bad_logic_loop: loop {
            count += 1;

            // The termination check is completed when starting
            let r1 = additional.send_open_signal();
            let r2 = additional.send_shutdown_signal();

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

            match additional
                .channels
                .ui_shutdown_conf
                .recv_timeout(Duration::from_millis(100))
            {
                Ok(_) => {
                    *GTK_SHUTDOWN_AT_END.lock().unwrap() = true;
                    gtk = Ok(());
                    break 'bad_logic_loop;
                }
                Err(e) => match e {
                    std::sync::mpsc::RecvTimeoutError::Timeout => {
                        // 10 seconds
                        if count > 100 {
                            gtk = Err(SettingsGtkShutdownErr::FailedWithinTimeLimit);

                            break 'bad_logic_loop;
                        }
                    }
                    std::sync::mpsc::RecvTimeoutError::Disconnected => {
                        gtk = Err(SettingsGtkShutdownErr::Disconnected(
                            std::sync::mpsc::RecvTimeoutError::Disconnected,
                        ));
                        break 'bad_logic_loop;
                    }
                },
            }
        }
    } else {
        gtk = Err(SettingsGtkShutdownErr::ThreadAlreadyTerminated);
    }

    let pw;

    if pw_1.is_ok() {
        match additional
            .channels
            .dbus_shutdown_conf
            .recv_timeout(Duration::from_secs(3))
        {
            Ok(_) => {
                *PIPEWIRE_SHUTDOWN_AT_END.lock().unwrap() = true;

                pw = Ok(())
            }
            Err(e) => pw = Err(PipewireShutdownErr::TimeoutOrTermination(e)),
        }
    } else {
        pw = Err(PipewireShutdownErr::ThreadAlreadyTerminated);
    }

    let full = ShutdownResult {
        gtk_settings_ui: gtk,
        pipewire_threads: pw,
    };

    let wrapped;

    if full.gtk_settings_ui.is_ok() && full.pipewire_threads.is_ok() {
        wrapped = Ok(full);
    } else {
        wrapped = Err(full);
    }

    println!("{:#?}", wrapped);

    ev.exit();

    wrapped
}
