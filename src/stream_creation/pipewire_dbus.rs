use ashpd::desktop::{
    PersistMode,
    screencast::{Screencast, SourceType},
};
use dbus::{
    Message, Path,
    arg::{self, Variant},
    blocking::Connection,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};

pub struct GnomePipewireWindowStream<'a> {
    node_id: u32,
    #[allow(unused)]
    window: gnome_window_calls::abstraction::Window,
    connection: dbus::blocking::Connection,
    // a session is created to launch streams
    session_path: dbus::Path<'a>,
    // when a stream is launched, it has a specific path to manage it
    stream_path: dbus::Path<'a>,
}

enum IntegerChoice {
    #[allow(unused)]
    U32(u32),
    U64(u64),
}

impl dbus::arg::Append for IntegerChoice {
    fn append_by_ref(&self, i: &mut arg::IterAppend) {
        match self {
            IntegerChoice::U64(v) => {
                i.append_variant(&dbus::Signature::make::<u64>(), move |i| i.append(v))
            }
            IntegerChoice::U32(v) => {
                i.append_variant(&dbus::Signature::make::<u32>(), move |i| i.append(v))
            }
        }
    }
}

impl dbus::arg::Arg for IntegerChoice {
    const ARG_TYPE: arg::ArgType = arg::ArgType::Variant;

    fn signature() -> dbus::Signature<'static> {
        dbus::Signature::make::<Variant<IntegerChoice>>()
    }
}

/// 0: hidden - cursor is not included in the stream
///
/// 1: embedded - cursor is included in the framebuffer
///
/// 2: metadata - cursor is included as metadata in the PipeWire stream
#[derive(Debug)]
pub enum CursorMode {
    /// 0: hidden - cursor is not included in the stream
    Hidden = 0,
    /// 1: embedded - cursor is included in the framebuffer
    Embedded = 1,
    /// 2: metadata - cursor is included as metadata in the PipeWire stream
    Metadata = 2,
}

pub trait PipewireStream {
    fn id(&self) -> u32;
}

pub struct FreeDesktopPipewireWindowStream {
    inner: ashpd::desktop::screencast::Stream,
}

impl PipewireStream for FreeDesktopPipewireWindowStream {
    fn id(&self) -> u32 {
        self.inner.pipe_wire_node_id()
    }
}

impl FreeDesktopPipewireWindowStream {
    pub fn create_stream(
        _window: &gnome_window_calls::abstraction::Window,
        cursor_mode: CursorMode,
    ) -> FreeDesktopPipewireWindowStream {
        let mode = match cursor_mode {
            CursorMode::Hidden => ashpd::desktop::screencast::CursorMode::Hidden,
            CursorMode::Embedded => ashpd::desktop::screencast::CursorMode::Embedded,
            CursorMode::Metadata => ashpd::desktop::screencast::CursorMode::Metadata,
        };

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();

        let stream = rt.block_on(async {
            let proxy = Screencast::new().await.unwrap();
            let session = proxy.create_session().await.unwrap();
            proxy
                .select_sources(
                    &session,
                    mode,
                    SourceType::Window | SourceType::Monitor,
                    false,
                    None,
                    PersistMode::DoNot,
                )
                .await
                .unwrap();

            let response = proxy
                .start(&session, None)
                .await
                .unwrap()
                .response()
                .unwrap();

            response.streams().into_iter().next().unwrap().clone()
        });

        FreeDesktopPipewireWindowStream { inner: stream }
    }
}

impl<'a> GnomePipewireWindowStream<'a> {
    pub fn create_stream(
        window: &gnome_window_calls::abstraction::Window,
        cursor_mode: CursorMode,
    ) -> GnomePipewireWindowStream<'a> {
        let conn = Connection::new_session().unwrap();

        let proxy = conn.with_proxy(
            "org.gnome.Mutter.ScreenCast",
            "/org/gnome/Mutter/ScreenCast",
            Duration::from_millis(5000),
        );

        let m: HashMap<String, Variant<String>> = HashMap::new();

        let session_path: (Path,) = proxy
            .method_call("org.gnome.Mutter.ScreenCast", "CreateSession", (m,))
            .unwrap();

        let session_path = session_path.0;

        let proxy = conn.with_proxy(
            "org.gnome.Mutter.ScreenCast",
            session_path.clone(),
            Duration::from_millis(5000),
        );

        // https://www.alteeve.com/w/List_of_DBus_data_types#INT16
        //
        // Refer to the above for determining the DBus type.
        /*
        <method name="RecordWindow">
            <arg name="properties" type="a{sv}" direction="in" />
            <arg name="stream_path" type="o" direction="out" />
        </method>

        * "window-id" (t): Id of the window to record.
        * "cursor-mode" (u): Cursor mode. Default: 'hidden' (see below)
                             Available since API version 2.
        * "is-recording" (b): Whether this is a screen recording. May be
                      be used for choosing panel icon.
                      Default: false. Available since API version 4.

        Available cursor mode values:

        0: hidden - cursor is not included in the stream
        1: embedded - cursor is included in the framebuffer
        2: metadata - cursor is included as metadata in the PipeWire stream


            */
        // https://gitlab.gnome.org/GNOME/gnome-shell/-/blob/92d3c6e051958b31151bf9538205a71cab6f70d7/data/dbus-interfaces/org.gnome.Mutter.ScreenCast.xml

        let mut m: HashMap<String, IntegerChoice> = HashMap::new();

        m.insert("window-id".into(), IntegerChoice::U64(window.id as u64));

        match cursor_mode {
            CursorMode::Hidden => {}
            value => {
                m.insert("cursor-mode".into(), IntegerChoice::U32(value as u32));
            }
        }

        let stream_path: (Path,) = proxy
            .method_call("org.gnome.Mutter.ScreenCast.Session", "RecordWindow", (m,))
            .unwrap();

        let stream_path: Path = stream_path.0;

        println!("Stream: {stream_path:?}");
        println!("Starting...");

        let stream_listening_proxy = conn.with_proxy(
            "org.gnome.Mutter.Stream",
            stream_path.clone(),
            Duration::from_millis(5000),
        );

        let waiting_for_pipewire: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
        let message_side = waiting_for_pipewire.clone();

        let _ = stream_listening_proxy.match_signal(
            move |pw_event: PipeWireStreamCreated, _: &Connection, message: &Message| {
                println!(
                    "A pipewire node_id was received: {}",
                    pw_event.pipewire_node_id
                );
                println!("{message:?}");

                *message_side.lock().unwrap() = Some(pw_event.pipewire_node_id);
                true
            },
        );

        let _: () = proxy
            .method_call("org.gnome.Mutter.ScreenCast.Session", "Start", ())
            .unwrap();

        {
            while None == *waiting_for_pipewire.lock().unwrap() {
                conn.process(Duration::from_millis(10)).unwrap();
            }
        }

        let pipewire_id: u32 = waiting_for_pipewire.lock().unwrap().unwrap();

        GnomePipewireWindowStream {
            node_id: pipewire_id,
            window: window.clone(),
            stream_path: stream_path,
            connection: conn,
            session_path,
        }
    }
}

impl<'a> PipewireStream for GnomePipewireWindowStream<'a> {
    fn id(&self) -> u32 {
        self.node_id
    }
}

impl<'a> Drop for GnomePipewireWindowStream<'a> {
    fn drop(&mut self) {
        let proxy = self.connection.with_proxy(
            "org.gnome.Mutter.ScreenCast",
            self.stream_path.clone(),
            Duration::from_millis(5000),
        );

        let _: Result<(), _> = proxy.method_call("org.gnome.Mutter.ScreenCast.Stream", "Stop", ());

        let proxy = self.connection.with_proxy(
            "org.gnome.Mutter.ScreenCast",
            self.session_path.clone(),
            Duration::from_millis(5000),
        );

        // I think this won't ever be successful because I think ending the stream with the previous call also
        // ends the session. Though, it seems more correct manually stop the session too.
        let _: Result<(), _> = proxy.method_call("org.gnome.Mutter.ScreenCast.Session", "Stop", ());
    }
}

#[derive(Debug)]
pub struct PipeWireStreamCreated {
    pub pipewire_node_id: u32,
}

impl arg::AppendAll for PipeWireStreamCreated {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.pipewire_node_id, i);
    }
}

impl arg::ReadAll for PipeWireStreamCreated {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(PipeWireStreamCreated {
            pipewire_node_id: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for PipeWireStreamCreated {
    const NAME: &'static str = "PipeWireStreamAdded";
    const INTERFACE: &'static str = "org.gnome.Mutter.ScreenCast.Stream";
}
