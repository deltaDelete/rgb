#[allow(dead_code)]
pub trait ForName: Sized {
    fn for_name(name: &str) -> Option<Self>;
    async fn for_name_async(name: &str) -> Option<Self> {
        Self::for_name(name)
    }
}