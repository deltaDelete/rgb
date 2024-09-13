use gtk::gdk::{Display, Monitor};
use gtk::prelude::{DisplayExt, IsA, ListModelExtManual, MonitorExt};
use gtk::Window;
use gtk4_layer_shell::LayerShell;

#[allow(dead_code)]
pub trait LayerShellExt: IsA<Window> {
    fn set_monitor_by_connector(&self, connector: &str) {
        if let Some(monitor) = Display::default()
            .unwrap()
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
        {
            self.set_monitor(&monitor);
        }
    }
}

impl<T: IsA<gtk::Window>> LayerShellExt for T {}
