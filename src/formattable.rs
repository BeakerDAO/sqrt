pub trait Formattable {
    fn format(&self) -> String;
}

impl Formattable for String {
    fn format(&self) -> String {
        self.to_lowercase().replace("_", " ")
    }
}

impl Formattable for str {
    fn format(&self) -> String {
        String::from(&self).to_lowercase().replace("_", " ")
    }
}
