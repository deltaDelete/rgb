use gtk::prelude::WidgetExt;
use relm4::prelude::AsyncComponentParts;
use relm4::prelude::SimpleAsyncComponent;

pub struct DateTime {
    label: String,
    format: String,
}

#[relm4::component(pub async)]
impl SimpleAsyncComponent for DateTime {
    type Input = ();
    type Output = ();
    type Init = ();

    view! {
        #[root]
        gtk::Box {
            add_css_class: "module",

            gtk::Label {
                #[watch]
                set_label: model.label.as_str(),
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: relm4::AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let time = chrono::Local::now();
        let format = "%H:%M:%S %d.%m.%Y";
        let label = time.format(format).to_string();
        let model = Self {
            format: format.to_string(),
            label,
        };
        let widgets = view_output!();

        let sender_clone = _sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
            loop {
                interval.tick().await; // Wait for the next tick
                sender_clone.input(());
            }
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
            &mut self,
            _message: Self::Input,
            _sender: relm4::AsyncComponentSender<Self>,
        ) -> () {
        match _message {
            () => self.update_time()
        };
    }
}

impl DateTime {
    fn update_time(&mut self) {
        self.label = chrono::Local::now().format(&self.format).to_string();
    }
}