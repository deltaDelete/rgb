pub trait KeyboardLayoutEx {
    fn find(display_name: &str) -> Option<xkb_data::KeyboardLayout> {
        if let Ok(layouts) = xkb_data::keyboard_layouts() {
            let x = layouts.layout_list.layout;
            // TODO find by description, show other name
            let result = x
                .iter()
                .find(|&it| it.description() == display_name)?
                .clone();
            Some(result)
        } else {
            None
        }
    }

    fn localized(&self) -> Option<String>;
}

impl KeyboardLayoutEx for xkb_data::KeyboardLayout {
    fn localized(&self) -> Option<String> {
        let result = language_tags::LanguageTag::parse(self.name()).ok()?;
        Some(match result.region() {
            None => result.primary_language().to_string(),
            Some(region) => region.to_string(),
        })
    }
}


#[cfg(test)]
mod tests {
    use log::{info, LevelFilter};
    use crate::ext::KeyboardLayoutEx;

    static INIT: std::sync::Once = std::sync::Once::new();

    pub fn init() {
        INIT.call_once(|| {
            // env_logger::builder()
            //     .target(env_logger::Target::Stdout)
            //     .init();
            env_logger::builder()
                .target(env_logger::Target::Stdout)
                .filter_module("regbar", LevelFilter::Debug)
                .init();
        });
    }

    #[test]
    fn test_find_layout() -> Result<(), &'static str> {
        init();
        let result = xkb_data::KeyboardLayout::find("Russian").expect("Wasn't found!");
        assert_eq!(result.name(), "ru");
        Ok(())
    }

    #[test]
    fn localize_layout() -> Result<(), &'static str> {
        init();
        let result = xkb_data::KeyboardLayout::find("Russian").expect("Wasn't found!");
        let localized = result.localized();
        info!("localized = {:?}", localized);
        assert_ne!(localized, None);
        Ok(())
    }
}