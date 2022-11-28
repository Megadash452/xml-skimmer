use std::collections::HashMap;
use xml_skimmer::{ParsedNode, SkimError};

#[test]
fn skim_xml() -> Result<(), SkimError> {
    let mut node_count = 0;
    
    xml_skimmer::skim_xml(include_str!("sample.xml"), HashMap::from([
        ("tag", |node: &ParsedNode| {
            println!("Call successful for {node}");
            node_count += 1;
        })
    ]))
}