pub mod selector;
use std::{collections::{HashMap, HashSet}, fmt::Display};
use crate::selector::{CommaSeparated, Selector};


/// Parse an xml source can call handler closures when a node that matches a selector is found.
pub fn skim_xml<F>(xml_src: &str, handlers: HashMap<&'static str, F>) -> Result<(), SkimError>
where F: FnMut(&ParsedNode) {
    let mut stack: Vec<ParsedNode> = vec![];
    // Node that this fn is working with. Will be pushed to stack if is an OPENING_NODE, and popped if is a CLOSING_NODE
    let mut current_node = ParsedNode::default();
    // Temporary attribute; will be added to the last ParsedNode
    let mut current_attr = Attr::default();
    let mut node_type = NodeType::None;
    // Whether the characters being read are appended to the tag, an attribute name, or an attribute value
    let mut writing_to = WriteTo::Content;

    // parse selector strings
    let mut handlers = handlers.into_iter().map(|(sel, fun)| {
        (sel.parse::<CommaSeparated<Selector>>().unwrap(), fun)
    }).collect::<Vec<(CommaSeparated<Selector>, F)>>();


    let mut iter = xml_src.chars();
    while let Some(character) = iter.next() {
        // Anything goes in a TextNode (except `<`)
        if writing_to == WriteTo::Content && character != '<' {
            // TODO: write text content
            // todo!("write text content");
            continue;
        }

        match character {
            // Creating an OPENING_NODE
            '<' => {
                node_type = NodeType::Opening;
                writing_to = WriteTo::Tag;

                /* Check if the next 3 characters are !-- to initiate a comment.
                   Save a slice of the remaining characters after !-- */
                if let Some(remaining) = iter.as_str().strip_prefix("!--") {
                    println!("Comment Start");
                    /* Look for the end-of-comment delimeter (-->) */
                    let remaining = match remaining.split_once("-->") {
                        Some((content, remaining)) => {
                            // print comment content
                            println!("    {content}");
                            remaining
                        }
                        // The rest of xml_src is the comment
                        None => return Err(SkimError::UnclosedComment(remaining.to_string()))
                    };

                    // skip the comment and its delimeters
                    iter = remaining.chars();
                    println!("Comment Stop");
                }
                // Treat prolog nodes <?xml?> as comments
                else if let Some(remaining) = iter.as_str().strip_prefix("?") {
                    println!("Prolog start");
                    // Question-mark (?) is used as a delimiter, look for the ending one
                    let remaining = match remaining.split_once("?>") {
                        Some((content, remaining)) => {
                            // print prolog content
                            println!("    {content}");
                            remaining
                        }
                        // The rest of xml_src is the comment
                        None => return Err(SkimError::UnclosedComment(remaining.to_string()))
                    };

                    // skip the prolog and its delimeter
                    iter = remaining.chars();
                    println!("Prolog Stop");
                }
            }
            // Change OPENING_NODE to CLOSING_NODE
            '/' => {
                /* Empty tag at this point means this is a regular closing node.
                   If tag has content it means this is a self-closing node */
                if current_node.tag.is_empty() {
                    node_type = NodeType::Closing;
                } else {
                    node_type = NodeType::SelfClosing;
                }
            }
            // Stop creating the OPENING_NODE or CLOSING_NODE. Then Push or Pop from stack
            '>' => {
                // Push any remaining attribute
                if !current_attr.name.is_empty() {
                    current_node.attributes.insert(current_attr.name, current_attr.value);
                }

                // Managing XML Stack
                match node_type {
                    // Doe something if a selector matches the current_node
                    NodeType::Opening | NodeType::SelfClosing => {
                        stack.push(current_node);
                        // Handlers: when a node has been parsed and some data needs to be read from it
                        // Check if any selector (keys in the HashMap) matches current_node
                        for (sel, handler) in handlers.iter_mut() {
                            if sel.match_node(&stack) {
                                handler(stack.last().unwrap());
                            }
                        }
                        // When is self-closing, node is pushed, matched, then removed.
                        if node_type == NodeType::SelfClosing {
                            stack.pop();
                        }
                    }
                    // Pop last ParsedNode.
                    NodeType::Closing =>
                        // Tag of last ParsedNode must be identical to the current/CLOSING_NODE
                        match stack.pop() {
                            Some(node) if current_node.tag == node.tag => {
                                // print!("{}", " ".repeat((stack.len() * INDENT_AMOUNT) as usize));
                                // println!("</\x1b[91m{}\x1b[0m>", node.tag);
                            }
                            Some(node) => return Err(SkimError::CantCloseNode(current_node.tag, Some(node))),
                            None => return Err(SkimError::CantCloseNode(current_node.tag, None))
                        },
                    // NodeType::None will not be reached here
                    NodeType::None => panic!("Found '>' with NodeType::None")
                }

                // Reset Values
                current_node = ParsedNode::default();
                current_attr = Attr::default();
                writing_to = WriteTo::Content;
                node_type = NodeType::None;
            }
            
            _ if character.is_whitespace() => {
                // Whitespace only matters in an OPENING_NODE
                if node_type == NodeType::Opening {
                    match writing_to {
                        // Switch from writing to tag -> writing to attr_name
                        WriteTo::Tag if !current_node.tag.is_empty() => writing_to = WriteTo::AttrName,
                        // Push attr (if name not empty) to current_node (In case of duplicate attr, the last one read will remain)
                        // Case of Boolean Attributes (e.g.: <tag attr1 attr2>)
                        WriteTo::AttrName => {
                            // Look for the equal sign (=) before hitting any other char (except whitespace)
                            while let Some(character) = iter.next() {
                                match character {
                                    // Equal sign (=) means to begin AttrVal
                                    '=' => {
                                        writing_to = WriteTo::AttrVal;
                                        break;
                                    }
                                    // Ignore whitespace
                                    ' ' | '\n' | '\t' => {  }
                                    // A different attribute has been reached
                                    _ => {
                                        // Only push attribute if it exists
                                        if !current_attr.name.is_empty() {
                                            // Attr will have an empty value
                                            current_node.attributes.insert(current_attr.name, String::new());
                                            current_attr = Attr::default();
                                        }
                                        // add this character to the new attribute, as it will be skipped by the iterator
                                        current_attr.name.push(character);
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Switch from writing to attr.name -> writing to attr.value
            '=' => {
                // = Only allowed to separate AttrName and AttrVal, when writing AttrVal, and text Content
                // WriteTo::AttrVal and WriteTo::Content will never be reached here
                if node_type == NodeType::Opening && writing_to == WriteTo::AttrName {
                    writing_to = WriteTo::AttrVal;
                } else {
                    return Err(SkimError::BadEqSign)
                }
            }
            // Switch from writing to attr.val -> writing to attr.name
            '"' | '\'' => {
                // Quotes (single or double) should only be used in AttrVal and text Content
                match writing_to  {
                    WriteTo::AttrVal => {
                        // AttrVal starts at the quote, and should end at the next quote of the same type (single or double)
                        // Start and end quotes are ignored
                        let remaining = match iter.as_str().split_once(character) {
                            Some((attr_val, remaining)) => {
                                // AttrVal is the slice before the end quote
                                current_node.attributes.insert(current_attr.name, String::from(attr_val));
                                remaining
                            }
                            None => return Err(SkimError::UnclosedString(current_attr.name, current_node))
                        };
                        // Finished reading AttrVal, proceed to next Attr
                        current_attr = Attr::default();
                        writing_to = WriteTo::AttrName;
                        // skip iteration of AttrVal; continue over the rest of the xml_src
                        iter = remaining.chars();
                    }
                    // WriteTo::Content will never be reached here
                    _ => return Err(SkimError::BadQuote)
                }
            }
            
            _ => {
                match writing_to {
                    WriteTo::Tag => current_node.tag.push(character),
                    WriteTo::AttrName => current_attr.name.push(character),
                    // WriteTo::AttrVal will never be reached here
                    // WriteTo::Content will never be reached here
                    _ => panic!("{writing_to:?} should have not been reached")
                }
            }    
        }
    }

    /* There should be no ParsedNodes left in the stack at this point.
       If there is, it means the xml is not written properly */
    if stack.len() > 0 {
        Err(SkimError::UnclosedNode)
    } else {
        Ok(())
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

#[derive(Debug, PartialEq, Eq)]
enum WriteTo {
    Tag, AttrName, AttrVal, Content
}

// struct TextNode {
//     content: String
// }

/// A pair of strings
#[derive(Default)]
pub struct Attr {
    pub name: String,
    pub value: String
}


#[derive(Debug, Default)]
pub struct ParsedNode {
    pub tag: String,
    pub attributes: HashMap<String, String>
}
impl ParsedNode {
    pub fn class_list(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            // Classes are separated by space
            Some(list) => list.split(' ').collect(),
            None => HashSet::new()
        }
    }
}
impl Display for ParsedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<\x1b[92m{tag}\x1b[0m \x1b[36m{attrs:?}\x1b[0m>", tag=self.tag, attrs=self.attributes)
    }
}


#[derive(Debug)]
pub enum SkimError {
    BadQuote,
    UnclosedNode,
    UnclosedComment(String),
    /// Contains [`Attr`]::name and [`ParsedNode`] that contains the [`Attr`].
    UnclosedString(String, ParsedNode),
    /// Conitans the attempted closing tag `</tag>` and the last [`ParsedNode`] in the stack.
    CantCloseNode(String, Option<ParsedNode>),
    BadEqSign,
}
impl Display for SkimError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadQuote => write!(f, "Quotes (single or double) not supposed to be here!"),
            Self::UnclosedNode => write!(f, "One or more Nodes were not closed"),
            Self::UnclosedComment(content) => write!(f, "Unclosed comment: -> {content}"),
            Self::UnclosedString(attr_name, node) => write!(f, "Missing closing quote (single or double) of attribute {attr_name} in node {node} (perhaps wrong quote was used to close)"),
            Self::CantCloseNode(closing_tag, Some(last_node)) => write!(f, "Rogue Closing_Node <{closing_tag}>, last ParsedNode is <{last_node}>"),
            Self::CantCloseNode(closing_tag, None) => write!(f, "Rogue Closing_Node <{closing_tag}>"),
            Self::BadEqSign => write!(f, "Equal_Sign (=) not supposed to be here!"),
        }
    }
}
