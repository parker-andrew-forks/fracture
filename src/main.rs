use crate::global_application_state::{GTK_SHUTDOWN_AT_END, PIPEWIRE_SHUTDOWN_AT_END, SAFE_MODE};
use crate::gpu_mirror_display::defaults::FP_ID;
use application_channel_creator::ApplicationChannelsCreator;
use ashpd::desktop::notification::{NotificationProxy, Priority};
use global_application_state::DESKTOP_ENV_IS_GNOME;
use gpu_mirror_display::event_loop::run_mirror_video_output_ui;
use gtk_user_interfaces::{
    install_ui::{self},
    settings_ui::run_settings_ui,
};
use std::env;
use stream_creation::start_mirror_stream::start_mirroring;

pub mod application_channel_creator;
pub mod global_application_state;
pub mod gpu_mirror_display;
pub mod gtk_user_interfaces;
pub mod shaders;
pub mod stream_creation;
pub mod ui_state;

pub async fn send_ash_notifcation(title: &str, msg: &str) -> ashpd::Result<()> {
    let proxy = NotificationProxy::new().await?;

    proxy
        .add_notification(
            FP_ID,
            ashpd::desktop::notification::Notification::new(&title)
                .body(msg)
                .priority(Priority::Urgent),
        )
        .await?;

    Ok(())
}

pub fn send_notifcation(title: &str, msg: &str) -> ashpd::Result<()> {
    let tokio = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    tokio.block_on(async { send_ash_notifcation(&title, &msg).await })
}

#[derive(Debug)]
pub struct JoinedAtEnd {
    pipewire: bool,
    gtk: bool,
}

fn main() {
    // This is a hack until I fix it... For my device, the GDK_BACKEND is set to x11 and I don't know why...
    //
    // If the GDK_BACKEND needs to be set then set the FORCE_OVERRIDE_GDK value in the environment and it won't be changed.
    {
        if let Err(_) = env::var("FORCE_OVERRIDE_GDK") {
            unsafe {
                env::remove_var("GDK_BACKEND");

                // The problem is that my environment is somehow setting the GDK_BACKEND=x11
                // when it supposed to be GDK_BACKEND=wayland. If it's changed to wayland, it works,
                // but, wayland isn't necessarily the default GDK would use. GTK attempts to detect the
                // best available otpion if it's not set, so by removing the environmental variable,
                // GTK will try to set the best option itself.

                // env::set_var("GDK_BACKEND", "wayland");
            }
        }
    }

    let (gpu, ui, dbus) = ApplicationChannelsCreator::channels();
    let (send_init_complete, init_complete) = std::sync::mpsc::channel::<()>();

    let gtk_user_interfaces_handle = std::thread::spawn(move || {
        if *DESKTOP_ENV_IS_GNOME {
            let _startup_result = install_ui::gtk_installer_launcher();
        }

        let _ = send_init_complete.send(());

        let conf = ui.shutdown_confirmed.clone();
        run_settings_ui(ui);
        let _ = conf.send(());
    });

    init_complete.recv().unwrap();

    let window_recording_handle = std::thread::spawn(|| {
        let focussed = {
            match env::var(SAFE_MODE) {
                Err(_) => {
                    let windows = match gnome_window_calls::abstraction::Windows::list() {
                        Ok(windows) => windows,
                        Err(e) => {
                            if *DESKTOP_ENV_IS_GNOME {
                                let msg = format!(
                                    "{} \r\n\r\n{e:?}\r\n\r\n{}",
                                    "Window access failed. If it normally works, try rebooting.",
                                    "The most common reason for this failure is that Gnome Shell Extensions were updated and Gnome Shell needs to restart before this extension can function normally again"
                                );

                                println!("{msg}");

                                // todo: fix broken notification. do not uncomment, it makes
                                // the app look like a virus.
                                //
                                // let _ = send_notifcation("Failed to start", &msg);
                            }

                            vec![]
                        }
                    };

                    let focussed = windows.into_iter().find(|v| v.cache.focus.unwrap_or(false));

                    focussed
                }
                Ok(_) => None,
            }
        };

        let conf = dbus.shutdown_confirmed.clone();
        start_mirroring(focussed, dbus);
        let _ = conf.send(());
    });

    run_mirror_video_output_ui(gpu).expect("The window should always successfully run.");

    let mut temp = JoinedAtEnd {
        pipewire: true,
        gtk: true,
    };

    if !*PIPEWIRE_SHUTDOWN_AT_END.lock().unwrap() {
        match window_recording_handle.is_finished() {
            true => println!("pw result: {:#?}", window_recording_handle.join()),
            false => {
                temp.pipewire = false;
                println!("the pw thread is being dropped without finishing.")
            }
        }
    } else {
        window_recording_handle.join().unwrap();
    }

    // if this is true, and we made it to here, it either panicked or it failed to shutdown within a time limit.
    if !*GTK_SHUTDOWN_AT_END.lock().unwrap() {
        match gtk_user_interfaces_handle.is_finished() {
            true => println!("gtk result: {:#?}", gtk_user_interfaces_handle.join()),
            false => {
                temp.gtk = false;
                println!("the gtk thread is being dropped without finishing.");
            }
        }
    } else {
        gtk_user_interfaces_handle.join().unwrap();
    }

    let temp2;

    if !temp.gtk || !temp.pipewire {
        temp2 = Err(temp);
    } else {
        temp2 = Ok(temp);
    }

    match &temp2 {
        Ok(_) => {
            println!("Successful shutdown.")
        }
        Err(_) => {
            println!("{:#?}", temp2);

            std::process::exit(0);
        }
    }
}
