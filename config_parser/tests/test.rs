use config_parser2::*;
use serde::Deserialize;

#[derive(ConfigParse, Deserialize, Default, Debug, PartialEq)]
struct A {
    pub a1: String,
    pub a2: u32,
    pub a3: bool,
    pub a4: Vec<B>,
    pub a5: C,
}

#[derive(ConfigParse, Deserialize, Default, Debug, PartialEq)]
struct B {
    #[serde(default)]
    pub b1: String,
    #[serde(default)]
    pub b2: String,
}

#[derive(ConfigParse, Deserialize, Default, Debug, PartialEq)]
struct C {
    pub c1: B,
    pub c2: bool,
    pub c3: bool,
}

#[derive(ConfigParse, Deserialize, Default, Debug, PartialEq)]
struct D {
    pub d1: Option<B>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use config_parser2::ConfigParser;

    #[test]
    fn option_test_none_to_none() {
        let mut value = D { d1: None };
        let toml = "".parse::<toml::Value>().unwrap();
        let expected_value = D { d1: None };

        value.parse(toml).unwrap();
        assert!(value == expected_value);
    }

    #[test]
    fn option_test_none_to_some() {
        let mut value = D { d1: None };
        let toml = "
[d1]
b1 = 'd1.b1'
b2 = 'd1.b2'
"
        .parse::<toml::Value>()
        .unwrap();
        let expected_value = D {
            d1: Some(B {
                b1: "d1.b1".to_owned(),
                b2: "d1.b2".to_owned(),
            }),
        };

        value.parse(toml).unwrap();
        assert!(value == expected_value);
    }

    #[test]
    fn option_test_some_to_none() {
        let mut value = D {
            d1: Some(B {
                b1: "d1.b1".to_owned(),
                b2: "d1.b2".to_owned(),
            }),
        };
        let toml = "".parse::<toml::Value>().unwrap();
        let expected_value = D {
            d1: Some(B {
                b1: "d1.b1".to_owned(),
                b2: "d1.b2".to_owned(),
            }),
        };

        value.parse(toml).unwrap();
        assert!(value == expected_value);
    }

    #[test]
    fn option_test_some_to_some() {
        let mut value = D {
            d1: Some(B {
                b1: "d1.b1".to_owned(),
                b2: "d1.b2".to_owned(),
            }),
        };
        let toml = "
[d1]
b1 = 'd1.b1_new'
b2 = 'd1.b2_new'
"
        .parse::<toml::Value>()
        .unwrap();
        let expected_value = D {
            d1: Some(B {
                b1: "d1.b1_new".to_owned(),
                b2: "d1.b2_new".to_owned(),
            }),
        };

        value.parse(toml).unwrap();
        assert!(value == expected_value);
    }

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
            a1: "a1".to_owned(),
            a2: 510,
            a3: false,
            a4: vec![B {
                b1: "a4.b1".to_owned(),
                b2: "a4.b2".to_owned(),
            }],
            a5: C {
                c1: B {
                    b1: "a5.c1.b1".to_owned(),
                    b2: "a5.c1.b2".to_owned(),
                },
                c2: true,
                c3: false,
            },
        };

        let toml = "
a1 = 'a1_new'
a2 = 150

[[a4]]
b2 = 'a4.b2'

[[a4]]
b1 = 'a4.b1'

[a5.c1]
b1 = 'a5.c1.b1_new'
b2 = 'a5.c1.b2_new'

[a5]
c2 = false
c3 = true
"
        .parse::<toml::Value>()
        .unwrap();

        let expected_value = A {
            a1: "a1_new".to_owned(),
            a2: 150,
            a3: false,
            a4: vec![
                B {
                    b1: "".to_owned(),
                    b2: "a4.b2".to_owned(),
                },
                B {
                    b1: "a4.b1".to_owned(),
                    b2: "".to_owned(),
                },
            ],
            a5: C {
                c1: B {
                    b1: "a5.c1.b1_new".to_owned(),
                    b2: "a5.c1.b2_new".to_owned(),
                },
                c2: false,
                c3: true,
            },
        };

        value.parse(toml).unwrap();
        assert!(value == expected_value);
    }
}
