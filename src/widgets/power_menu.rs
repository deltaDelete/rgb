use crate::ext::ParentFor;
use crate::new_stateless_value_action;
use gtk::prelude::*;
use log::{error, info};
use relm4::actions::{RelmAction, RelmActionGroup};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};

pub struct PowerMenu;

#[relm4::component(pub)]
impl SimpleComponent for PowerMenu {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        #[root]
        gtk::Button {
            add_css_class: "module",
            add_css_class: "hoverable",
            add_css_class: "power-menu",
            set_icon_name: "system-shutdown",
            set_parent_for: popover = &gtk::PopoverMenu::from_model(Some(&power_menu)),
            connect_clicked[popover] => move |_btn| {
                popover.popup();
            }
        }
    }

    menu! {
        power_menu: {
            "Shutdown" => Exec("systemctl shutdown".into()),
            "Reboot" => Exec("systemctl reboot".into()),
            "Sleep" => Exec("hyprctl dispatch dpms off".into()),
            "Lock" => Exec("hyprlock --immediate".into()),
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self;
        let widgets = view_output!();

        let action: RelmAction<Exec> =
            RelmAction::new_with_target_value(|_action, value: String| {
                info!("{:?}", value);
                match std::process::Command::new("bash")
                    .arg("-c")
                    .arg(value.clone())
                    .spawn()
                {
                    Ok(_std) => {
                        info!("Successfully run command: {:?}", value);
                    }
                    Err(err) => {
                        error!("Error running command: {:?}\n{}", value, err);
                    }
                };
            });

        let mut group = RelmActionGroup::<ShellActionGroup>::new();
        group.add_action(action);
        group.register_for_widget(&widgets.popover);

        ComponentParts { model, widgets }
    }
}

relm4::new_action_group!(ShellActionGroup, "power");
new_stateless_value_action!(Exec, ShellActionGroup, "exec", String);
