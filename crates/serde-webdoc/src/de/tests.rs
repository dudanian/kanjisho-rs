use super::*;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};
use web_sys::{DomParser, SupportedType};

wasm_bindgen_test_configure!(run_in_browser);

macro_rules! parse_from_string {
    ($text:ident) => {{
        let parser = DomParser::new().unwrap();
        let doc = parser
            .parse_from_string(&$text, SupportedType::TextXml)
            .unwrap();
        from_doc(&doc)
    }};
}

mod positive {
    use super::*;

    #[wasm_bindgen_test]
    fn deserialize_pos_int() {
        let text = "<test>12345</test>";
        let res: Result<i32> = parse_from_string!(text);
        assert_eq!(res, Ok(12345));
    }

    #[wasm_bindgen_test]
    fn deserialize_neg_int() {
        let text = "<test>-12345</test>";
        let res: Result<i32> = parse_from_string!(text);
        assert_eq!(res, Ok(-12345));
    }

    #[wasm_bindgen_test]
    fn deserialize_string() {
        let text = "<test>hello</test>";
        let res: Result<String> = parse_from_string!(text);
        assert_eq!(res, Ok(String::from("hello")));
    }

    #[wasm_bindgen_test]
    fn deserialize_struct() {
        let text = "<test>12345</test>";
        let res: Result<i32> = parse_from_string!(text);
        assert_eq!(res, Ok(12345));
    }

    #[wasm_bindgen_test]
    fn deserialize_seq() {
        let text = "<test>12345</test>";
        let res: Result<i32> = parse_from_string!(text);
        assert_eq!(res, Ok(12345));
    }

    #[wasm_bindgen_test]
    fn deserialize_opt_some() {
        let text = "<test>hello</test>";
        let res: Result<Option<String>> = parse_from_string!(text);
        assert_eq!(res, Ok(Some(String::from("hello"))));
    }

    // can't test opt-none until we have a struct
    // since we are defining some as the presence of an element

    #[wasm_bindgen_test]
    fn deserialize_unit() {
        let text = "<test></test>";
        let res: Result<()> = parse_from_string!(text);
        assert_eq!(res, Ok(()));
    }
}

mod negative {
    use super::*;

    #[wasm_bindgen_test]
    fn deserialize_invalid_i32() {
        let text = "<test>12hello34</test>";
        let res: Result<i32> = parse_from_string!(text);
        assert_eq!(res, Err(Error::ParseError));
    }

    #[wasm_bindgen_test]
    fn deserialize_overflow() {
        let text = "<test>12345</test>";
        let res: Result<i8> = parse_from_string!(text);
        assert_eq!(res, Err(Error::ParseError));
    }

    #[wasm_bindgen_test]
    fn deserialize_unit() {
        let text = "<test>hello</test>";
        let res: Result<()> = parse_from_string!(text);
        assert_eq!(res, Err(Error::NonEmptyUnit));
    }
}

mod structed {
    use super::*;

    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct Nest1 {
        nest: Nest2,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Nest2 {
        list: Vec<Nest3>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Nest3 {
        num: i32,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Test {
        #[serde(default)]
        name: String,
        #[serde(default)]
        num: i32,
        #[serde(default)]
        list1: Vec<String>,
        list2: Option<Vec<String>>,
    }

    impl Test {
        fn basic(name: &str, num: i32) -> Self {
            Test {
                name: name.into(),
                num,
                list1: vec![],
                list2: None,
            }
        }

        fn vec(list1: Vec<&str>, list2: Option<Vec<&str>>) -> Self {
            Test {
                name: String::default(),
                num: i32::default(),
                list1: list1.iter().map(|s| (*s).to_owned()).collect(),
                list2: list2.map(|l| l.iter().map(|s| (*s).to_owned()).collect()),
            }
        }
    }

    #[wasm_bindgen_test]
    fn deserialize_basic() {
        let text = "<test><name>hello</name><num>1234</num></test>";
        let res: Result<Test> = parse_from_string!(text);
        assert_eq!(res, Ok(Test::basic("hello", 1234)));
    }

    #[wasm_bindgen_test]
    fn deserialize_vecs() {
        let text = "<test><list1>hi</list1><list1>there</list1><list2>test</list2><list2>me</list2></test>";
        let res: Result<Test> = parse_from_string!(text);
        assert_eq!(
            res,
            Ok(Test::vec(vec!["hi", "there"], Some(vec!["test", "me"])))
        );
    }

    #[wasm_bindgen_test]
    fn deserialize_vec1() {
        let text = "<test><list1>hi</list1><list1>there</list1></test>";
        let res: Result<Test> = parse_from_string!(text);
        assert_eq!(res, Ok(Test::vec(vec!["hi", "there"], None)));
    }

    #[wasm_bindgen_test]
    fn deserialize_vec2() {
        let text = "<test><list2>test</list2><list2>me</list2></test>";
        let res: Result<Test> = parse_from_string!(text);
        assert_eq!(res, Ok(Test::vec(vec![], Some(vec!["test", "me"]))));
    }

    #[wasm_bindgen_test]
    fn deserialize_struct() {
        let text = "<test><name>hello</name><num>1234</num></test>";
        let res: Result<Test> = parse_from_string!(text);
        assert_eq!(res, Ok(Test::basic("hello", 1234)));
    }

    #[wasm_bindgen_test]
    fn deserialize_nested_struct() {
        let text = "<test><nest><list><num>1</num></list><list><num>2</num></list><list><num>3</num></list></nest></test>";
        let res: Result<Nest1> = parse_from_string!(text);
        assert_eq!(
            res,
            Ok(Nest1 {
                nest: Nest2 {
                    list: vec![Nest3 { num: 1 }, Nest3 { num: 2 }, Nest3 { num: 3 }]
                }
            })
        );
    }
}
