use std::collections::HashMap;
use xml_skimmer::selector;

#[test]
fn skim_xml() {
    let mut node_count = 0;
    
    xml_skimmer::skim_xml(include_str!("sample.xml"), HashMap::from([
        ("tag", |node: &selector::ParsedNode| {
            println!("Call successful for {}", node);
            node_count += 1;
        })
    ]));
}