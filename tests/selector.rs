use std::collections::{HashMap, HashSet};
use xml_skimmer::{ParsedNode, selector::{CommaSeparated, Selector}};

#[test]
fn matching() {
    let node = ParsedNode{
        tag: String::from("tag"),
        attributes: HashMap::from([
            (String::from("class"), String::from("class cls c")),
            (String::from("id"),    String::from("id")),
            (String::from("attr"),  String::from("val"))
        ])
    };

    assert!("tag"                          .parse::<CommaSeparated<Selector>>().unwrap().match_node(&node));
    assert!("tag2, tag"                    .parse::<CommaSeparated<Selector>>().unwrap().match_node(&node));
    assert!(".cls"                         .parse::<CommaSeparated<Selector>>().unwrap().match_node(&node));
    assert!("#id"                          .parse::<CommaSeparated<Selector>>().unwrap().match_node(&node));
    assert!("[attr]"                       .parse::<CommaSeparated<Selector>>().unwrap().match_node(&node));
    assert!("[attr=val]"                   .parse::<CommaSeparated<Selector>>().unwrap().match_node(&node));
    // all combined
    assert!("tag#id.class.cls.c[attr=val]" .parse::<CommaSeparated<Selector>>().unwrap().match_node(&node));
}

#[test]
fn all_selector_tokens() {
    assert_eq!("tag".parse(),
        Ok(Selector {
            tag: "tag".to_string().into(),
            ..Default::default()
        })
    );

    assert_eq!("#id".parse(),
        Ok(Selector {
            id: "id".to_string().into(),
            ..Default::default()
        })
    );

    assert_eq!(".class".parse(),
        Ok(Selector {
            classes: HashSet::from(["class".to_string()]),
            ..Default::default()
        })
    );

    assert_eq!("[ attr ]".parse(),
        Ok(Selector {
            attributes: HashMap::from([("attr".to_string(), None)]),
            ..Default::default()
        })
    );

    assert_eq!("[ attr = val ]".parse(),
        Ok(Selector {
            attributes: HashMap::from([("attr".to_string(), "val".to_string().into())]),
            ..Default::default()
        })
    );

    assert_eq!("[ attr = 'val' ]".parse(),
        Ok(Selector {
            attributes: HashMap::from([("attr".to_string(), "val".to_string().into())]),
            ..Default::default()
        })
    );


    // all combined
    assert_eq!(
        "tag#id.class.cls.c[attr][attr1=val1][attr2=\"val2\"][attr3='val3'][ attr4 = val4 ][ attr5 = 'val5' ]".parse(),
        Ok(Selector {
            tag: "tag".to_string().into(),
            id: "id".to_string().into(),
            classes: HashSet::from(["class".to_string(), "cls".to_string(), "c".to_string()]),
            attributes: HashMap::from([
                ("attr".to_string(), None),
                ("attr1".to_string(), "val1".to_string().into()),
                ("attr2".to_string(), "val2".to_string().into()),
                ("attr3".to_string(), "val3".to_string().into()),
                ("attr4".to_string(), "val4".to_string().into()),
                ("attr5".to_string(), "val5".to_string().into()),
            ])
        })
    );
}

#[test]
fn selector_erorrs() {
    use xml_skimmer::selector::SelectorParseError as Error;

    assert_eq!("".parse::<Selector>(),                Err(Error::EmptyString));
    assert_eq!(" tag ".parse::<Selector>(),           Err(Error::WhiteSpace));
    assert_eq!("#id1#id2".parse::<Selector>(),        Err(Error::MultipleIDs));
    assert_eq!(".class.class".parse::<Selector>(),    Err(Error::DuplicateClass));
    assert_eq!("[attr][attr]".parse::<Selector>(),    Err(Error::DuplicateAttr));
    assert_eq!("[ ]".parse::<Selector>(),             Err(Error::EmptyAttrName));
    assert_eq!("[ = ]".parse::<Selector>(),           Err(Error::EmptyAttrName));
    assert_eq!("[ = val ]".parse::<Selector>(),       Err(Error::EmptyAttrName));
    assert_eq!("tag&".parse::<Selector>(),            Err(Error::UnknownPrefix));
    assert_eq!("[ attr = 'val ]".parse::<Selector>(), Err(Error::UnclosedString));
    assert_eq!("[ attr = val ".parse::<Selector>(),   Err(Error::UnclosedBracket));
    assert_eq!("[ attr = ".parse::<Selector>(),       Err(Error::UnclosedBracket));
    assert_eq!("[ attr = ]".parse::<Selector>(),      Err(Error::BadChar));
    assert_eq!("[attr='val'' ]".parse::<Selector>(),  Err(Error::BadChar));
    assert_eq!("tag.class=.cls".parse::<Selector>(),  Err(Error::BadChar));
    assert_eq!("tag.class].cls".parse::<Selector>(),  Err(Error::BadChar));
}
