pub trait ConfigParser {
    fn parse(&mut self, value: toml::Value);
}

#[macro_export]
macro_rules! config_parser_impl {
    ($($t:ty),+) => {
        $(
            impl ConfigParser for $t {
                fn parse(&mut self, value: toml::Value) {
                    *self = value.try_into::<$t>().unwrap();
                }
            }
        )*
    };
}

impl<T: ConfigParser + Default> ConfigParser for Vec<T> {
    fn parse(&mut self, value: toml::Value) {
        if let toml::Value::Array(array) = value {
            *self = array
                .into_iter()
                .map(|e| {
                    let mut v = T::default();
                    v.parse(e);
                    v
                })
                .collect::<Vec<_>>();
        }
    }
}

config_parser_impl!(String, usize, u64, u32, bool);
