use std::{collections::HashMap, fmt::Display};

fn main() {
    parse_xml(include_str!("sample.xml"))
}



pub fn parse_xml(xml_src: &str) {
    let mut stack: Vec<ParsedNode> = vec![];
    // Node parser is working with. Will be pushed to stack if is an OPENING_NODE, and popped if is a CLOSING_NODE
    let mut current_node = ParsedNode::new();
    let mut current_attr = Attr::new(); // temporary attribute; will be added to the last ParsedNode
   
    let mut node_type = NodeType::None;

    // Whether the characters being read are appended to the tag, an attribute name, or an attribute value
    let mut writing_to = WriteTo::Content;

    // * for debug only, remove after
    let mut indent_level: u32 = 0;
    const INDENT_AMOUNT: u32 = 4;


    for character in xml_src.chars() {
        // Anything goes in an attribute value (except `"` or `'`)
        if writing_to == WriteTo::AttrVal && character != '"' && character != '\'' {
            current_attr.value.push(character);
            continue;
        }
        // Anything goes in a TextNode (except `<`)
        else if writing_to == WriteTo::Content && character != '<' {
            // TODO: write text content
            continue;
        }

        match character {
            // Creating an OPENING_NODE
            '<' => {
                node_type = NodeType::Opening;
                writing_to = WriteTo::Tag;

                // TODO: Detect comment node
            }
            // Change OPENING_NODE to CLOSING_NODE
            '/' => {
                /* Empty tag at this point means this is a regular closing node.
                   If tag has content it means this is a self-closing node */
                if current_node.tag == "" {
                    node_type = NodeType::Closing;
                } else {
                    node_type = NodeType::SelfClosing;
                }
            }
            // Stop creating the OPENING_NODE or CLOSING_NODE. Then Push or Pop from stack
            '>' => {
                // Push any remaining attribute
                if current_attr.name != "" {
                    // If at this point the attr has a non-empty value, it means the string was not closed correctly
                    if current_attr.value == "" {
                        current_node.attributes.insert(current_attr.name, current_attr.value);
                    } else {
                        panic!("The string of attr `{}` in Node {} was not closed correctly", current_attr.name, current_node)
                    }
                }

                // Handlers
                match node_type {
                    NodeType::Opening | NodeType::SelfClosing => {
                        // TODO: check if tag and attrs match the selector, then call handler
                    }
                    _ => {}
                }

                // Managing XML Stack
                match node_type {
                    // Push ParsedNode to stack
                    NodeType::Opening => {
                        print!("{}", " ".repeat((indent_level * INDENT_AMOUNT) as usize));
                        println!("{}", &current_node);
                        indent_level += 1;

                        stack.push(current_node);
                    }
                    NodeType::SelfClosing => {
                        print!("{}", " ".repeat((indent_level * INDENT_AMOUNT) as usize));
                        println!("<\x1b[92m{}\x1b[0m \x1b[36m{:?}\x1b[0m\x1b[91m/\x1b[0m>", current_node.tag, current_node.attributes)
                    }
                    // Pop last ParsedNode.
                    NodeType::Closing => {
                        // decrement will panic if there are more OPENING_NODEs than CLOSING_NODEs
                        indent_level -= 1;
                        print!("{}", " ".repeat((indent_level * INDENT_AMOUNT) as usize));
                        println!("</\x1b[91m{}\x1b[0m>", current_node.tag);
                        
                        // Tag of last ParsedNode must be identical to the current/CLOSING_NODE
                        if current_node.tag == stack.last().unwrap().tag {
                            stack.pop();
                        } else {
                            panic!("Rogue Closing_Node <{}>, last ParsedNode is <{}>", current_node.tag, stack.last().unwrap());
                        }
                    }
                    _ => {}
                }

                // Reset Values
                current_node = ParsedNode::new();
                current_attr = Attr::new();
                writing_to = WriteTo::Content;
                node_type = NodeType::None;
            }
            
            ' ' | '\n' => {
                // Whitespace only matters in an OPENING_NODE
                if node_type == NodeType::Opening {
                    match writing_to {
                        // Switch from writing to tag -> writing to attr_name
                        WriteTo::Tag => writing_to = WriteTo::AttrName,
                        // Push attr (if name not empty) to current_node (In case of duplicate attr, the last one read will remain)
                        // Case of Boolean Attributes (e.g.: <tag attr1 attr2>)
                        WriteTo::AttrName => {
                            if current_attr.name != "" {
                                current_node.attributes.insert(current_attr.name, current_attr.value);
                                current_attr = Attr::new();
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Switch from writing to attr.name -> writing to attr.value
            '=' => {
                // = Only allowed to separate AttrName and AttrVal, when writing AttrVal, and text Content
                match node_type {
                    NodeType::Opening =>
                        match writing_to {
                            WriteTo::AttrName => writing_to = WriteTo::AttrVal,
                            // WriteTo::AttrVal will never be reached here
                            _ => panic!("Equal_Sign (=) not supposed to be here!")
                        }
                    NodeType::Closing => panic!("Equal_Sign (=) not supposed to be here!"),
                    _ => {}
                }
            }
            // Switch from writing to attr.val -> writing to attr.name
            '"' | '\'' => {
                let char_0 = current_attr.value.chars().nth(0);
                // quotes (single or double) should only be used in AttrVal and text Content
                match writing_to  {
                    WriteTo::AttrVal =>
                        // attr.val must have at least 1st char as a quote
                        // Both are either single (') or double (") quote
                        if char_0 == Some(character) {
                            // The first " must be removed, since its only purpose is being a delimeter
                            current_attr.value.remove(0);
                            writing_to = WriteTo::AttrName;
                            // Push attr to current_node (In case of duplicate attr, the last one read will remain)
                            current_node.attributes.insert(current_attr.name, current_attr.value);
                            // reset
                            current_attr = Attr::new();
                        }
                        // char_0 and character are different
                        else if char_0 == Some('"') ||
                                char_0 == Some('\'') ||
                        // Otherwise (if empty) it indicates that parser just started reading AttrVal
                                current_attr.value == "" {
                            /* Put the quote as 1st char for the next time parser encounters the same type of quote,
                            in which case it means the end of the AttrVal. */
                            current_attr.value.push(character);
                        }
                        // If attr_val is not empty and 1st char is not either type of quote, it means an error occurred
                        else {
                            panic!("Something went wrong reading value of attr {} in {} node", current_attr.name, current_node);
                        }
                    // WriteTo::Content will never be reached here
                    _ => panic!("Quotes (single or double) not suppoed to be here!")
                }
            }
            
            // TODO: add support for comment node
            _ => {
                match writing_to {
                    WriteTo::Tag => current_node.tag.push(character),
                    WriteTo::AttrName => current_attr.name.push(character),
                    // WriteTo::AttrVal will never be reached here
                    _ => todo!("TextNode Content")
                }
            }    
        }
    }

    /* There should be no ParsedNodes left in the stack at this point.
       If there is, it means the xml is not written properly */
    if stack.len() > 0 {
        panic!("One or more Nodes were not closed:\n{:#?}", stack);
    }
}

#[derive(PartialEq, Eq)]
enum NodeType {
    /* OPENING_NODEs contain all of a ParsedNode's information like `tag` and `attributes`.
       Are created when parser encounters the pattern "<"
       Once an OPENING_NODE is finished reading (because it will encounter a '>'), a ParsedNode will be pushed to the stack */
    Opening,
    /* CLOSING_NODEs represent just a tag.
       Are created when the parser encounters the pattertn "</".
       Once a CLOSING_NODE is finished reading, a ParsedNode will be popped from the stack */
    Closing,
    /* Similar to OPENING_NODEs, but will not be pushed to the stack.
       Are created when parser encounters the patternn "/" within an OPENING_NODE, but node already has a tag */
    SelfClosing,
    // Not creating a node, mostly used for ignoring characters or creating text node
    None
}

#[derive(PartialEq, Eq)]
enum WriteTo {
    Tag, AttrName, AttrVal, Content
}

// struct TextNode {
//     content: String
// }

// A pair of strings
struct Attr {
    name: String,
    value: String
}
impl Attr {
    fn new() -> Self {
        Self{ name: String::new(), value: String::new() }
    }
}

#[derive(Debug)]
struct ParsedNode {
    tag: String,
    attributes: HashMap<String, String>
}
impl ParsedNode {
    fn new() -> Self {
        Self{ tag: String::new(), attributes: HashMap::new() }
    }
}
impl Display for ParsedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<\x1b[92m{}\x1b[0m \x1b[36m{:?}\x1b[0m>", self.tag, self.attributes)
    }
}