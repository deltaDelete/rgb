use gio::SimpleAction;
use gtk::gio;
use gtk::prelude::{FromVariant, StaticVariantType, ToVariant};
use relm4::actions::{ActionName, RelmAction};
use std::marker::PhantomData;

#[macro_export]
/// Create a new type that implements [`ActionName`] without state but with value.
macro_rules! new_stateless_value_action {
    ($vis:vis $ty:ident, $group:ty, $name:expr, $value:ty) => {
        $vis struct $ty;

        impl relm4::actions::ActionName for $ty {
            type Group = $group;
            type Target = $value;
            type State = ();

            const NAME: &'static str = $name;
        }
    };
}

#[allow(dead_code)]
trait StatelessValueAction<Name: ActionName>
where
    Name::State: ToVariant + FromVariant,
    Name::Target: ToVariant + FromVariant,
{
    fn new_stateless_with_target_value<Callback: Fn(&gio::SimpleAction, Name::Target) + 'static>(
        start_value: &Name::State,
        callback: Callback,
    ) -> Self;
}

impl<Name: ActionName> StatelessValueAction<Name> for RelmAction<Name>
where
    Name::State: ToVariant + FromVariant,
    Name::Target: ToVariant + FromVariant,
{
    fn new_stateless_with_target_value<Callback: Fn(&SimpleAction, Name::Target) + 'static>(
        _start_value: &Name::State,
        callback: Callback,
    ) -> Self {
        let ty = Name::Target::static_variant_type();

        let action = gio::SimpleAction::new(Name::NAME, Some(&ty));

        action.connect_activate(move |action, variant| {
            let value = variant.unwrap().get().unwrap();

            callback(action, value);
        });

        #[repr(C)]
        struct RelmActionFake<Name: ActionName> {
            name: PhantomData<Name>,
            action: gio::SimpleAction,
        }

        let internal: RelmActionFake<Name> = RelmActionFake {
            name: PhantomData,
            action,
        };

        unsafe { std::mem::transmute(internal) }
    }
}
