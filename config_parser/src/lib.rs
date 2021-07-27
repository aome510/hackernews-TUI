pub use anyhow::Result;

pub trait ConfigParser {
    fn parse(&mut self, value: toml::Value) -> Result<()>;
}

pub use config_parser_derive::ConfigParse;

#[macro_export]
macro_rules! config_parser_impl {
    ($($t:ty),+) => {
        $(
            impl ConfigParser for $t {
                fn parse(&mut self, value: toml::Value) -> Result<()> {
                    *self = value.try_into::<$t>()?;
                    Ok(())
                }
            }
        )*
    };
}

impl<'de, T> ConfigParser for Vec<T>
where
    T: serde::de::Deserialize<'de>,
{
    fn parse(&mut self, value: toml::Value) -> Result<()> {
        if let toml::Value::Array(array) = value {
            let result: Result<Vec<_>> =
                array.into_iter().map(|e| Ok(e.try_into::<T>()?)).collect();
            match result {
                Err(err) => Err(err),
                Ok(value) => {
                    *self = value;
                    Ok(())
                }
            }
        } else {
            Err(anyhow::anyhow!(
                "config parsing error: expect a TOML::Array, receive {:#?}",
                value
            ))
        }
    }
}

config_parser_impl!(
    String, usize, u128, u64, u32, u16, u8, isize, i128, i64, i32, i16, i8, f64, f32, bool, char
);
