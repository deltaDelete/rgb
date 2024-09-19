use app_module::AppModel;
use gtk::gio::ApplicationFlags;
use gtk::glib::{LogLevel, LogWriterOutput};
use gtk::prelude::{ApplicationCommandLineExt, ApplicationExt};
use gtk::{gdk, gio, glib, StyleContext};
use log::{error, info, log};
use relm4::once_cell::sync::Lazy;
use relm4::RelmApp;

mod app_module;
mod ext;
mod macros;
mod widgets;
mod workers;

const APP_ID: &str = "ru.deltadelete.rgb";

static STYLE: Lazy<gio::File> = Lazy::new(|| {
    let path = std::path::Path::new("./res/style.css");
    gio::File::for_path(path)
});

fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .target(env_logger::Target::Stdout)
        .init();
    info!("Hello, world!");
    let app = RelmApp::new(APP_ID);
    {
        let gtk_app = relm4::main_application();
        let flags = gtk_app.flags();
        gtk_app.set_flags(flags | ApplicationFlags::HANDLES_COMMAND_LINE);
        gtk_app.add_main_option(
            "reload-style",
            glib::Char::from(0x72),
            glib::OptionFlags::NONE,
            glib::OptionArg::None,
            "Reload css of running instance",
            None,
        );
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_file(&*STYLE);

        #[allow(deprecated)]
        StyleContext::add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &css_provider,
            800,
        );
        gtk_app.connect_command_line(move |application, options| {
            match options.options_dict().lookup::<bool>("reload-style") {
                Ok(Some(true)) => {
                    css_provider.load_from_file(&*STYLE);
                    info!("CSS reloaded!");
                }
                Err(e) => {
                    error!("{e}");
                }
                _ => {}
            }
            application.activate();
            -1
        });
    }
    gtk::glib::set_prgname(Some(APP_ID));
    redirect_glib_logs();
    initialize_icons();
    app.run_async::<AppModel>(())
}

fn initialize_icons() {
    relm4_icons::initialize_icons();
    if let Some(display) = gtk::gdk::Display::default() {
        let theme = gtk::IconTheme::for_display(&display);
        theme.add_resource_path("/ru/deltadelete/rgb/icons");
    }
}

fn redirect_glib_logs() {
    gtk::glib::log_set_writer_func(|level, x| {
        let level = match level {
            LogLevel::Error => log::Level::Error,
            LogLevel::Critical => log::Level::Error,
            LogLevel::Warning => log::Level::Warn,
            LogLevel::Message => log::Level::Trace,
            LogLevel::Info => log::Level::Info,
            LogLevel::Debug => log::Level::Debug,
        };
        let domain = x
            .iter()
            .find(|&it| it.key() == "GLIB_DOMAIN")
            .and_then(|it| it.value_str());
        let message = x
            .iter()
            .find(|&it| it.key() == "MESSAGE")
            .and_then(|it| it.value_str());

        if let Some(domain) = domain {
            if let Some(message) = message {
                log!(target: "gtk", level, "[{}] {}", domain, message);
            }
        } else if let Some(message) = message {
            log!(target: "gtk", level, "{}", message)
        }
        LogWriterOutput::Handled
    });
}
