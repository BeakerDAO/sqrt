#[macro_export]
macro_rules! call_args {
    () => (
        vec![]
    );

     ($( $x:expr ),*) => {{
        let mut temp_vec = vec![];
            $(
                temp_vec.push(Box::new($x));
            )*
        Some(temp_vec)
    }};
}
