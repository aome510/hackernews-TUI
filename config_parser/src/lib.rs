pub trait ConfigParser {
    fn parse(&mut self, value: toml::Value);
}

pub use config_parser_derive::ConfigParse;

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

config_parser_impl!(
    String, usize, u128, u64, u32, u16, u8, isize, i128, i64, i32, i16, i8, f64, f32, bool, char
);
