use gtk::prelude::IsA;
use gtk::Widget;

#[allow(dead_code)]
pub trait OnSelf {
    fn on_self<F: Fn(&Self)>(&self, f: F);
}

impl<T> OnSelf for T
where
    T: IsA<Widget>,
{
    fn on_self<F: Fn(&Self)>(&self, f: F) {
        f(self);
    }
}

