use gtk4 as gtk;
use gtk4::{
    ApplicationWindow, CssProvider,
    gdk::Display,
    gio::{
        ApplicationFlags,
        prelude::{ApplicationExt, ApplicationExtManual},
    },
    glib::{self, ControlFlow},
    prelude::{BoxExt, ButtonExt, GtkWindowExt, WidgetExt, WidgetExtManual},
};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use crate::gpu_mirror_display::defaults::APPLICATION_NAME;

#[derive(Clone, Debug)]
pub struct StartupResults {
    pub(crate) extensions_enabled: (bool, Option<String>),
    pub(crate) window_calls_installed: (bool, Option<String>),
    // application_installed: (bool, Option<String>),
    pub(crate) shortcut_set: (bool, Option<String>),
}

const SHORTCUT_NAME: &'static str = &APPLICATION_NAME;

/// Checks for mostly required installation requirements and providing a report for usage with the launcher
fn startup_checks() -> StartupResults {
    let extensions_enabled =
        gnome_window_calls::install::gnome_shell_extensions_are_enabled().is_ok();
    let window_calls_is_installed = gnome_window_calls::install::extension_is_enabled().is_ok();

    // The keyboard shortcut can be wrong, but by searching for a named shortcut it allows the user to change
    // the binding while still reporting it as bound. Alternatively, this could check for custom bindings that
    // execute this binary, but this will work well enough.
    let keyboard_binding_set = gnome_custom_keybindings::get_bindings()
        .into_iter()
        .any(|binding| binding.name == SHORTCUT_NAME);

    StartupResults {
        extensions_enabled: (extensions_enabled, None),
        window_calls_installed: (window_calls_is_installed, None),
        // application_installed: (false, None),
        shortcut_set: (keyboard_binding_set, None),
    }
}

#[derive(Clone, Debug)]
pub(crate) struct StartupState {
    pub(crate) results: StartupResults,
    pub(crate) redraw_requested: bool,
    pub(crate) should_shutdown: bool,
    pub(crate) failed: bool,
}

#[derive(Debug, Clone)]
pub enum EnableGnomeExtensionsErr {
    ImNotOkay,
    ClosedConfirmWindow,
}

trait UseState: Send + Sync {
    fn execute(&self, state: Rc<RefCell<StartupState>>);
}

impl UseState for Box<dyn Fn(Rc<RefCell<StartupState>>) + Send + Sync> {
    fn execute(&self, state: Rc<RefCell<StartupState>>) {
        self(state);
    }
}

fn fix_closure(f: impl Fn(Rc<RefCell<StartupState>>) + Send + Sync + 'static) -> Box<dyn UseState> {
    converter(Box::new(f))
}

fn converter(v: Box<dyn Fn(Rc<RefCell<StartupState>>) + Send + Sync>) -> Box<dyn UseState> {
    Box::new(v)
}

fn rebuild(state: Rc<RefCell<StartupState>>, scheduler: CustomGtkScheduler) -> gtk::Box {
    let base = gtk::Box::builder()
        .valign(gtk4::Align::Start)
        .spacing(10)
        .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .margin_top(10)
        .orientation(gtk4::Orientation::Vertical)
        .build();

    let extensions_enabled_box = gtk::Box::builder()
        .valign(gtk4::Align::Start)
        .spacing(10)
        .margin_end(10)
        .margin_start(10)
        // .margin_top(10)
        .orientation(gtk4::Orientation::Horizontal)
        .build();

    let ext = gtk::Label::builder().use_markup(true).build();

    extensions_enabled_box.append(&ext);

    let result = if state.borrow().results.extensions_enabled.0 {
        extensions_enabled_box.add_css_class("ok");
        extensions_enabled_box.add_css_class("pad_box");

        format!(
            "<span size=\"large\"><b>Gnome Shell Extensions Enabled {}</b></span>\r\n",
            "✅"
        )
    } else {
        extensions_enabled_box.add_css_class("err");
        extensions_enabled_box.add_css_class("pad_box");

        let enable_extensions_btn = gtk::Button::builder()
            .label("Enable")
            .hexpand(true)
            .halign(gtk4::Align::End)
            .build();

        extensions_enabled_box.append(&enable_extensions_btn);

        let state_copy2 = state.clone();

        enable_extensions_btn.connect_clicked(move |btn| {
            btn.add_css_class("warn");
            btn.set_sensitive(false);

            let state = state_copy2.clone();

            let enable_result = gnome_window_calls::install::enable_gnome_shell_extensions();

            if let Err(err) = enable_result {
                let debug = format!("{:#?}", err);

                let temp: &mut StartupState = &mut state.borrow_mut();

                temp.redraw_requested = true;
                temp.results.extensions_enabled.1 = Some(debug);

                return;
            }

            #[allow(deprecated)]
            let confirm_dia = gtk::Dialog::builder().title("").build();
            let dialog_content_box = gtk::Box::builder()
                .valign(gtk4::Align::Start)
                .spacing(10)
                .margin_end(10)
                .margin_start(10)
                .margin_bottom(10)
                .orientation(gtk4::Orientation::Vertical)
                .build();

            dialog_content_box.set_width_request(1920 / 4);

            let fix = Instant::now();

            let time_display = gtk::Label::builder().build();
            time_display.set_use_markup(true);
            time_display.set_markup("<span size=\"xx-large\">Are you OK?</span>");

            let no = gtk::Label::builder()
                .justify(gtk4::Justification::Center)
                .build();

            dialog_content_box.append(&time_display);

            let confirm_ok = gtk::Button::builder().label("Submit").build();

            dialog_content_box.append(&confirm_ok);
            dialog_content_box.append(&no);

            confirm_dia.set_child(Some(&dialog_content_box));

            #[allow(deprecated)]
            confirm_dia.show();

            let state_closure = state_copy2.clone();
            let state_closure_2 = state_copy2.clone();

            let closed_by_confirmed_ok = Rc::new(RefCell::new(false));
            let close_copy = closed_by_confirmed_ok.clone();

            confirm_dia.connect_close_request(move |_| {
                let confirmed_ok = { *close_copy.borrow() };

                if !confirmed_ok {
                    let _ = gnome_window_calls::install::disable_gnome_shell_extensions();
                }

                let temp: &mut StartupState = &mut state_closure_2.borrow_mut();

                let err: Result<(), _> = Err(EnableGnomeExtensionsErr::ClosedConfirmWindow);

                temp.redraw_requested = true;
                temp.results.extensions_enabled.0 = false;
                temp.results.extensions_enabled.1 = Some(format!("{:#?}", err));

                glib::Propagation::Proceed
            });

            let close_copy = closed_by_confirmed_ok.clone();

            confirm_dia.add_tick_callback(move |c_dia, _| {
                let help = Duration::from_secs(15).checked_sub(fix.elapsed());

                if let Some(one) = help {
                    // I was considering text as it counted down, but changed my mind.
                    let start = match one.as_millis() {
                        _ => "",
                    };

                    no.set_markup(&format!("{}{}", one.as_secs(), start));
                } else {
                    no.set_text("NOW");

                    {
                        {
                            *close_copy.borrow_mut() = false;
                        }

                        // calling `c_dia.close()` triggers an event that disables gnome_shell_extensions when close_copy (from above) is checked
                        //  let _ = gnome_window_calls::install::disable_gnome_shell_extensions();
                        c_dia.close()
                    }

                    {
                        // todo: It might be a good idea to show errors to the user.
                        // let _ = gnome_window_calls::install::disable_gnome_shell_extensions();

                        let temp: &mut StartupState = &mut state_closure.borrow_mut();

                        let err: Result<(), _> = Err(EnableGnomeExtensionsErr::ImNotOkay);

                        temp.redraw_requested = true;
                        temp.results.extensions_enabled.0 = false;
                        temp.results.extensions_enabled.1 = Some(format!("{:#?}", err));
                    }

                    return ControlFlow::Break;
                }

                ControlFlow::Continue
            });

            let state_copy = state.clone();

            confirm_ok.connect_clicked(move |_| {
                {
                    *closed_by_confirmed_ok.borrow_mut() = true;
                }

                confirm_dia.close();

                let temp: &mut StartupState = &mut state_copy.borrow_mut();

                // rerun the startup checks because it's possible for window calls to already be installed.
                temp.results = startup_checks();

                temp.redraw_requested = true;
            });
        });

        format!(
            "<span size=\"large\"><b>Gnome Shell Extensions Enabled {}</b></span>\r\n{}",
            "❌", "Gnome Shell Extensions need to be enabled before this application will work."
        )
    };

    ext.set_markup(&result);

    base.append(&extensions_enabled_box);

    match &&(state.borrow().results.extensions_enabled) {
        &(false, Some(error)) => {
            let text = gtk::TextBuffer::builder().text(error.clone()).build();

            let err = gtk::TextView::builder()
                .buffer(&text)
                .margin_start(10)
                .margin_end(10)
                .build();

            err.set_visible(true);

            base.append(&err);
        }
        _ => {}
    }

    let window_calls_installed_box = gtk::Box::builder()
        .valign(gtk4::Align::Start)
        .spacing(10)
        // .margin_bottom(10)
        .margin_end(10)
        .margin_start(10)
        .orientation(gtk4::Orientation::Horizontal)
        .build();

    let ext = gtk::Label::builder().use_markup(true).build();

    window_calls_installed_box.append(&ext);

    let result2 = if state.borrow().results.window_calls_installed.0 {
        window_calls_installed_box.add_css_class("ok");
        window_calls_installed_box.add_css_class("pad_box");

        format!(
            "<span size=\"large\"><b>Window Calls Installed {}</b></span>\r\n",
            "✅"
        )
    } else {
        window_calls_installed_box.add_css_class("err");
        window_calls_installed_box.add_css_class("pad_box");

        let install_btn = gtk::Button::builder()
            .label("Install")
            .hexpand(true)
            .halign(gtk4::Align::End)
            .build();

        if !state.borrow().results.extensions_enabled.0 {
            install_btn.set_sensitive(false);
        }

        window_calls_installed_box.append(&install_btn);

        install_btn.connect_clicked(move |btn| {
            btn.add_css_class("warn");
            btn.set_sensitive(false);

            let scheduler = scheduler.clone();

            // This is just to prevent gtk from hanging while it installs
            std::thread::spawn(move || {
                let scheduler = scheduler.clone();
                let result = gnome_window_calls::install::install_window_calls();

                let _ = scheduler.schedule_execution(move |state| {
                    let temp: &mut StartupState = &mut state.borrow_mut();
                    temp.redraw_requested = true;

                    if let Err(e) = &result {
                        let debug = format!("{:#?}", e);

                        temp.results.window_calls_installed.1 = Some(debug);
                    } else {
                        temp.results.window_calls_installed.0 = true;
                    }
                });
            });
        });

        let l1 = "<span size=\"large\"><b>Window Calls Installed ❌</b></span>";
        let l2 = "The window-calls Gnome Shell Extension needs to be installed.";

        format!("{}\r\n{}", l1, l2)
    };

    ext.set_markup(&result2);
    base.append(&window_calls_installed_box);

    match &state.borrow().results.window_calls_installed {
        (false, Some(error)) => {
            let err = gtk::TextView::builder()
                .buffer(&gtk::TextBuffer::builder().text(error).build())
                // .wrap_mode(gtk4::WrapMode::Word)
                .margin_start(10)
                .margin_end(10)
                .build();

            base.append(&err);
        }
        _ => {}
    }

    let keyboard_shortcut_box = gtk::Box::builder()
        .valign(gtk4::Align::Start)
        .spacing(10)
        .margin_end(10)
        .margin_start(10)
        .orientation(gtk4::Orientation::Horizontal)
        .build();

    let ext = gtk::Label::builder().use_markup(true).build();

    keyboard_shortcut_box.append(&ext);

    let result2 = if state.borrow().results.shortcut_set.0 {
        keyboard_shortcut_box.add_css_class("ok");
        keyboard_shortcut_box.add_css_class("pad_box");

        format!(
            "<span size=\"large\"><b>Keyboard Shortcut {}</b></span>\r\n",
            "✅"
        )
    } else {
        keyboard_shortcut_box.add_css_class("warn");
        keyboard_shortcut_box.add_css_class("pad_box");

        let enable_keyboard_shortcut_btn = gtk::Button::builder()
            .label("Enable")
            .hexpand(true)
            .halign(gtk4::Align::End)
            .build();

        keyboard_shortcut_box.append(&enable_keyboard_shortcut_btn);

        let l1 = "<span size=\"large\"><b>Keyboard Shortcut Missing ⚠️</b></span>";

        let shortcut = "Ctrl + Super + Shift + S";

        let l2 = format!(
            "The preferred keyboard shortcut <span size='small'>{}</span> is missing.",
            shortcut
        );

        let s = state.clone();

        enable_keyboard_shortcut_btn.connect_clicked(move |btn| {
            btn.add_css_class("warn");

            let temp: &mut StartupState = &mut s.borrow_mut();

            let defined_bindings: Vec<_> = gnome_custom_keybindings::get_bindings()
                .into_iter()
                .map(|v| v.name.clone())
                .collect();

            println!("{:?}", defined_bindings);

            let mut binding_is_good = true;

            if !defined_bindings.contains(&SHORTCUT_NAME.to_string()) {
                #[cfg(not(feature = "flatpak"))]
                let path = format!("/todo/fix/my/path");

                #[cfg(feature = "flatpak")]
                let path = format!(
                    "flatpak run {}",
                    crate::gpu_mirror_display::defaults::FP_ID
                );

                let _result = gnome_custom_keybindings::add_binding(
                    "<Shift><Control><Super>s",
                    &path,
                    SHORTCUT_NAME,
                );

                match &_result {
                    Ok(_) => {}
                    Err(e) => {
                        binding_is_good = false;

                        let text = e.clone();

                        temp.redraw_requested = true;
                        temp.results.shortcut_set = (false, Some(format!("{}", text)));
                    }
                }

                println!("{:?}", _result);
            }

            if binding_is_good {
                temp.redraw_requested = true;
                temp.results.shortcut_set = (true, None);
            }
        });

        format!("{}\r\n{}", l1, l2)
    };

    ext.set_markup(&result2);
    base.append(&keyboard_shortcut_box);

    match &&(state.borrow().results.shortcut_set) {
        &(false, Some(error)) => {
            let text = gtk::TextBuffer::builder().text(error.clone()).build();

            let err = gtk::TextView::builder()
                .buffer(&text)
                .margin_start(10)
                .margin_end(10)
                .build();

            err.set_visible(true);

            base.append(&err);
        }
        _ => {}
    }

    let complete_setup = gtk::Button::builder().label("Complete Setup").build();

    {
        let temp: &StartupResults = &state.borrow().results;

        if temp.extensions_enabled.0 && temp.window_calls_installed.0 {
            complete_setup.set_sensitive(true);
        } else {
            complete_setup.set_sensitive(false);
        }
    }

    complete_setup.connect_clicked(move |_| {
        state.borrow_mut().should_shutdown = true;
    });

    base.append(&complete_setup);

    base
}

#[derive(Clone, Debug)]
struct CustomGtkScheduler {
    handle: std::sync::mpsc::Sender<Box<dyn UseState>>,
}

impl CustomGtkScheduler {
    /// Schedules a closure to run on the tracked state within a Gtk tick loop. This is intended for use from another
    /// thread that needs to modify the state easiy
    pub fn schedule_execution(
        &self,
        f: impl Fn(Rc<RefCell<StartupState>>) + Send + Sync + 'static,
    ) -> Result<(), std::sync::mpsc::SendError<Box<dyn UseState>>> {
        self.handle.send(fix_closure(f))
    }
}

pub(crate) fn gtk_installer_launcher() -> StartupState {
    let checks: StartupResults = startup_checks();

    let full_state = Rc::new(RefCell::new(StartupState {
        results: checks.clone(),
        redraw_requested: false,
        should_shutdown: false,
        failed: true,
    }));

    if checks.extensions_enabled.0 && checks.window_calls_installed.0 {
        let mut temp = full_state.borrow().clone();
        temp.failed = false;

        return temp;
    }

    // installer
    {
        let application = gtk4::Application::builder()
            .flags(ApplicationFlags::NON_UNIQUE)
            .application_id("com.example.FirstGtkApp")
            .build();

        let full_state = full_state.clone();

        application.connect_activate(move |app| {
            let provider = CssProvider::new();

            let css = include_str!("../../css/border.css");

            provider.load_from_string(&css);

            gtk::style_context_add_provider_for_display(
                &Display::default().expect("Could not connect to a display."),
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            let window = ApplicationWindow::builder()
                .application(app)
                .title("Installer")
                .build();

            let (send, recv) = std::sync::mpsc::channel::<Box<dyn UseState>>();

            let scheduler = CustomGtkScheduler {
                handle: send.clone(),
            };

            let data = rebuild(full_state.clone(), scheduler.clone());

            window.set_child(Some(&data));

            let state_copy = full_state.clone();

            window.add_tick_callback(move |w, _| {
                if let Ok(value) = recv.try_recv() {
                    value.execute(state_copy.clone());
                }

                if state_copy.borrow().redraw_requested {
                    let data = rebuild(state_copy.clone(), scheduler.clone());
                    w.set_child(Some(&data));

                    state_copy.borrow_mut().redraw_requested = false;
                }

                if state_copy.borrow().should_shutdown {
                    let state: &mut StartupState = &mut state_copy.borrow_mut();

                    let results: &StartupResults = &state.results;

                    let did_setup_fail =
                        !(results.extensions_enabled.0 && results.window_calls_installed.0);

                    state.failed = did_setup_fail;

                    w.close();

                    return ControlFlow::Break;
                }

                ControlFlow::Continue
            });

            window.present();
        });

        application.run();
    }

    let after: StartupState = { full_state.borrow().clone() };

    // println!("{after:#?}");

    after
}
