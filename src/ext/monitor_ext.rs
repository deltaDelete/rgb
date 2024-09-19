use gtk::gdk::{Display, Monitor};
use gtk::prelude::{DisplayExt, IsA, ListModelExtManual};
use gtk::prelude::MonitorExt as _MonitorExt;

pub trait MonitorExt: IsA<gtk::gdk::Monitor> {
    fn for_connector(connector: &str) -> Option<gtk::gdk::Monitor> {
        Display::default()?
            .monitors()
            .iter::<Monitor>()
            // HINT for myself: flatten is equavalent of filter(Result::is_ok).map(Result::unwrap)
            .flatten()
            .find(|it| {
                if let Some(monitor) = it.connector() {
                    return monitor == connector;
                }
                false
            })
    }

    fn first() -> Monitor {
        Display::default().unwrap()
            .monitors()
            .iter::<Monitor>()
            .flatten().find(|_| true).unwrap()
    }
}

impl MonitorExt for Monitor {}