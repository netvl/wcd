
macro_rules! simple_error {
    ($name:ident, $description:expr) => {
        #[derive(Debug)]
        struct $name;

        impl ::std::error::Error for $name {
            fn description(&self) -> &str {
                $description
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                f.write_str(self.description())
            }
        }
    };
}
