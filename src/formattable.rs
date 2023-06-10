pub trait Formattable {
    fn format(&self) -> String;
}

impl<T> Formattable for T
where
    T: ToString,
{
    fn format(&self) -> String {
        self.to_string().to_lowercase().replace("_", " ")
    }
}
