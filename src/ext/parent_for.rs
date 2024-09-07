use gtk::prelude::{IsA, WidgetExt};
use gtk::Widget;

pub trait ParentFor {
    fn set_parent_for(&self, widget: &impl IsA<Widget>);
}

impl<T> ParentFor for T
where
    T: IsA<Widget>,
{
    fn set_parent_for(&self, widget: &impl IsA<Widget>) {
        widget.set_parent(self)
    }
}
