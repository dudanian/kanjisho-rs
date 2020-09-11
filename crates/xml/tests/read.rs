
use xml::read::Parser;
use xml::xml::*;

#[test]
fn element() {
    let text = "<test attr=\"val\" other = 'val' >hello</test>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![
                Attr {
                    name: String::from("attr"),
                    value: String::from("val")
                },
                Attr {
                    name: String::from("other"),
                    value: String::from("val")
                },
            ]
        })
    );
    assert_eq!(
        reader.token().unwrap(),
        Token::CharData(String::from("hello"))
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}

#[test]
fn autoclose() {
    let text = "<test attr='val'/>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![Attr {
                name: String::from("attr"),
                value: String::from("val")
            },]
        })
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}

#[test]
fn comment() {
    let text = "<test><!-- comment --></test>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![]
        })
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}

#[test]
fn proc_inst() {
    let text = "<?inst dostuff?><test/>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::ProcInst(ProcInst {
            target: String::from("inst"),
            inst: String::from("dostuff")
        })
    );
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![]
        })
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}

#[test]
fn xml_decl() {
    let text = "<?xml version=\"1.5\" encoding='uTf-8'?><test/>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![]
        })
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}

#[test]
fn doctype() {
    let text = "<!DOCTYPE test [  ]><test/>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![]
        })
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}

#[test]
fn char_ref() {
    let text = "<test>&#12486;&#x30B9;&#12488;&#x3002;</test>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![]
        })
    );
    assert_eq!(
        reader.token().unwrap(),
        Token::CharData(String::from("テスト。"))
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}

#[test]
fn entity_ref() {
    let text = "<test>this &gt; that &lt; those</test>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![]
        })
    );
    assert_eq!(
        reader.token().unwrap(),
        Token::CharData(String::from("this > that < those"))
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}

#[test]
fn cdata() {
    let text = "<test><![CDATA[</>>&le;\r\n<]]></test>";
    let mut reader = Parser::from_reader(text.as_bytes());
    assert_eq!(
        reader.token().unwrap(),
        Token::StartTag(StartTag {
            name: String::from("test"),
            attrs: vec![]
        })
    );
    assert_eq!(
        reader.token().unwrap(),
        Token::CharData(String::from("</>>&le;\n<"))
    );
    assert_eq!(reader.token().unwrap(), Token::EndTag(String::from("test")));
    assert_eq!(reader.token().unwrap(), Token::EndOfFile);
}
