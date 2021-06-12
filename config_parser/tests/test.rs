use config_parser2::*;
use serde::Deserialize;

#[derive(ConfigParse, Deserialize, Default, Debug, PartialEq)]
struct A {
    pub field_1: String,
    pub field_2: u32,
    pub field_3: bool,
    pub field_4: Vec<B>,
    pub field_5: C,
}

#[derive(ConfigParse, Deserialize, Default, Debug, PartialEq)]
struct B {
    pub field_1: String,
    pub field_2: String,
}

#[derive(ConfigParse, Deserialize, Default, Debug, PartialEq)]
struct C {
    pub field_1: B,
    pub field_2: bool,
    pub field_3: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use config_parser2::ConfigParser;

    #[test]
    fn simple_test() {
        let value = "a = ['b', 'c', 'd']".parse::<toml::Value>().unwrap();
        let mut a: Vec<String> = vec![];
        a.parse(value["a"].clone()).unwrap();
        assert!(a.len() == 3);
    }

    #[test]
    fn complex_test() {
        let mut value = A {
            field_1: "field_1".to_owned(),
            field_2: 510,
            field_3: false,
            field_4: vec![B {
                field_1: "b_field_1".to_owned(),
                field_2: "b_field_2".to_owned(),
            }],
            field_5: C {
                field_1: B {
                    field_1: "cb_field_1".to_owned(),
                    field_2: "cb_field_2".to_owned(),
                },
                field_2: true,
                field_3: false,
            },
        };

        let toml = "
field_1 = 'new_field_1'
field_2 = 150

[[field_4]]
field_2 = 'new_field_2'

[[field_4]]
field_1 = 'new_field_1'

[field_5.field_1]
field_1 = 'new_field_1'
field_2 = 'new_field_2'

[field_5]
field_2 = false
field_3 = true
"
        .parse::<toml::Value>()
        .unwrap();

        let expected_value = A {
            field_1: "new_field_1".to_owned(),
            field_2: 150,
            field_3: false,
            field_4: vec![
                B {
                    field_1: "".to_owned(),
                    field_2: "new_field_2".to_owned(),
                },
                B {
                    field_1: "new_field_1".to_owned(),
                    field_2: "".to_owned(),
                },
            ],
            field_5: C {
                field_1: B {
                    field_1: "new_field_1".to_owned(),
                    field_2: "new_field_2".to_owned(),
                },
                field_2: false,
                field_3: true,
            },
        };

        value.parse(toml).unwrap();
        assert!(value == expected_value);
    }
}
