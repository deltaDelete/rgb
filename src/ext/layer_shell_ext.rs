use gtk4_layer_shell::LayerShell;
use gtk::gdk::{Display, Monitor};
use gtk::prelude::{DisplayExt, IsA, ListModelExtManual, MonitorExt};
use gtk::Window;

pub trait LayerShellExt: IsA<Window> {
    fn set_monitor_by_connector(&self, connector: &str) {
        if let Some(monitor) = Display::default()
            .unwrap()
            .monitors()
            .iter::<Monitor>()
            .map(|it| it.ok())
            .filter(|it| match it {
                None => false,
                Some(_) => true
            })
            .map(|it| it.unwrap())
            .find(|it| {
                if let Some(monitor) = it.connector() {
                    return monitor.to_string() == connector
                }
                false
            }) {
            self.set_monitor(&monitor);
        }
    }
}

impl<T: IsA<gtk::Window>> LayerShellExt for T {}