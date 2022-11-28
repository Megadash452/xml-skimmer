use std::collections::HashMap;
use xml_skimmer::{ParsedNode, selector::match_to_node};

#[test]
fn css_selectors() {
    let node = ParsedNode{
        tag: String::from("tag"),
        attributes: HashMap::from([
            (String::from("class"), String::from("class cls c")),
            (String::from("id"),    String::from("id")),
            (String::from("attr"),  String::from("val"))
        ])
    };

    assert_eq!(match_to_node(&node, "tag"), true);
    assert_eq!(match_to_node(&node, "tag2, tag"), true);
    assert_eq!(match_to_node(&node, ".cls"), true);
    assert_eq!(match_to_node(&node, "#id"), true);
    assert_eq!(match_to_node(&node, "[attr]"), true);
    assert_eq!(match_to_node(&node, "[attr=val]"), true);
    assert_eq!(match_to_node(&node, "tag#id.class.cls.c[attr=val]"), true);        
}