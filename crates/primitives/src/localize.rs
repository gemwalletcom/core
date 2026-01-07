pub trait Localize {
    fn localize(&self, locale: &str) -> String;
}
