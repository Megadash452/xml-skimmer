use std::collections::HashMap;
use xml_skimmer::{ParsedNode, SkimError};

#[test]
fn benchmark() -> Result<(), SkimError> {
    // xml_skimmer::skim_xml(include_str!("benchmark.xml"), HashMap::new());
    Ok(())
}

#[test]
fn closures() -> Result<(), SkimError> {
    xml_skimmer::skim_xml(include_str!("benchmark.xml"), HashMap::from([
        ("depth", |node: &ParsedNode| {
            println!("Call successful for {node}");
        })
    ]))
}