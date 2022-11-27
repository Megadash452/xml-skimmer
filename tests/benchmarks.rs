use std::collections::HashMap;
use xml_skimmer::selector;

#[test]
fn benchmark() {
    // xml_skimmer::skim_xml::<
    //     selector::ParsedNode
    // >(include_str!("benchmark.xml"), HashMap::new());
}

#[test]
fn closures() {
    xml_skimmer::skim_xml(include_str!("benchmark.xml"), HashMap::from([
        ("depth", |node: &selector::ParsedNode| {
            println!("Call successful for {node}");
        })
    ]));
}