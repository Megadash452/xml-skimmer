use std::{collections::HashMap, fmt::Display};

/// Takes a css selector and splits it into subselectors which are comma-separated
fn separate_commas(selector: &str) -> Vec<&str> {
    // Stores the subselectors which will be returned
    let mut rtrn: Vec<&str> = vec![];

    // Commas inside strings (delimited by " or ') will be ignored
    let mut in_str = false;

    // start and end indices of the subselector &str
    let mut start = 0;
    let mut end = 0;

    for character in selector.chars() {
        match character {
            // Toggle in_str state
            '"' | '\'' => in_str = !in_str,
            ',' => {
                // Commas in strings are ignored
                if !in_str {
                    // Trim trailing whitespace before pushing
                    rtrn.push(selector[start..end].trim_matches(' '));
                    start = end + 1;
                }
            }
            _ => {}
        }

        end += 1;
    }

    // Push the last subselector if it exists
    let sub = selector[start..end].trim_matches(' ');
    if !sub.is_empty() {
        rtrn.push(sub);
    }

    rtrn
}

/// Get the slice that represents the tag in a css selector, and also the slice after
/// - e.g.: `tag#id.cls[attr=val]` -> (`tag`, `#id.cls[attr=val]`)
fn split_tag(selector: &str) -> (&str, &str) {
    // The end index of the tag slice
    let mut end = 0;

    for character in selector.chars() {
        match character {
            ' ' | '\t' | '\n' => {
                eprintln!("No whitespace allowed in selectors");
                return ("", "")
            }
            // Split tag at the selector objects (cls, id, attr respectively)
            '.' | '#' | '[' => {
                return (&selector[..end], &selector[end..])
            }
            _ => { }
        }

        end += 1;
    }

    // The entire selector is the tag
    return (selector, "")
}


/// Match a css selector to a node. Only supports `tag`, `#id`, `.class`, `[attr]`, `[attr="val"]`
/// - The `tag` should be the first part of the selector, and is optional.
/// - Then anything preceeded with `#` is an id. Looks for attribute `id` in node.
/// - Then anything preceeded with `.` is a class. Looks for attribute `class` in node.
/// - Then anything wrapped in `[]` is any other attribute.
///   - `[attr]` checks if an attribute exists at all in node.
///   - `[attr="val"]` or checks that an attribute has the specified value in node.
///     - Aliases: `[attr='val']`, `[attr=val]`
/// - Group multiple selectors into one by separating them with commas (,). If any of the selectors match the node, this fn will return true
///   - e.g.: `tag.cls, tag2.cls2`
/// <hr><br>
/// Not allowed: empty selector, and Combinators (e.g.: `tag tag2` or `tag > tag2`)
pub fn match_to_node(node: &impl ParseNode, selector: &str) -> bool {
    if selector.is_empty() {
        eprintln!("Empty selectors are not valid");
        return false
    }

    // Match each subselector to node
    'selectors: for selector in separate_commas(selector) {
        // The tag of the selector, and rest of the selector
        let (tag, rest) = split_tag(selector);
        // Iterate over the characters after the tag of the selector
        let mut iter = rest.chars();
        // Tells if this tag matches the node's tag
        let mut tag_match = false;

        // Match tag to node
        // When selector doesn't have tag (i.e. tag is empty) it is considered as matching because the node tag is being ignored
        if tag.is_empty() || tag == node.tag() {
            tag_match = true;
        }
        
        // The start and end of a selector object (class, id, attr)
        let mut start: usize = 0;
        let mut end: usize = 1;
        // Get the class attribute
        let classlist = node.class_list();

        // The first character woulc be a ., #, or [, but it should be skipped because we want to look for the next one
        iter.next();
        // Find class, id, other attributes
        for character in &mut iter {
            match character {
                '.' | '#' | '[' => {
                    // If any attribute does not match (i.e. this fn returns false), then this subselector does not match
                    if !match_sel_obj_to_node(node, &rest[start..end], &classlist) {
                        // Move on to the next selector
                        continue 'selectors;
                    }
                    start = end;
                }
                _ => { }
            }
            end += 1;
        }

        // When reached the end of selector, do one more for the last obj
        if tag_match && match_sel_obj_to_node(node, &rest[start..], &classlist) {
            return true
        }
    }

    false
}


/// Check if part of a css selector matches an attribute in node
/// - E.g: Check if `obj = ".cls"` matches in `Node{ _, attrs: {"class": "... cls ..."} }`
fn match_sel_obj_to_node(node: &impl ParseNode, obj: &str, classlist: &Vec<&str>) -> bool{
    // When obj is empty it is considered as matching because the selector ignores the node attributes
    if obj.is_empty() {
        return true
    }

    // First char of obj tells if it is class, id, or attr
    match obj.chars().nth(0) {
        // Match selector classs with one of node classes
        Some('.') => {
            if !classlist.contains(&&obj[1..]) {
                return false
            }
        }
        // Match selector id with node id
        Some('#') => {
            // Check if node has attribute named "id"
            match node.attributes().get("id") {
                Some(id) => {
                    // Check that id attr has specific value
                    if id != &obj[1..] {
                        return false
                    }
                }
                None => return false
            }
        }
        // Match selector attr with node attr
        Some('[') =>
            // Close this attr part of selector
            match obj[1..].split_once(']') {
                Some((attr, _)) =>
                    // Separate attr_name and attr_val
                    match attr.split_once('=') {
                        Some((mut attr_name, mut attr_val)) => {
                            // Trim trailing whitespace from attr_name
                            attr_name = attr_name.trim_matches(' ');

                            // Check if attr exists at all (with any value) in node
                            match node.attributes().get(attr_name) {
                                // Check if attr exists with specific value in node
                                Some(val) => {
                                    // Trim trailing whitespace from attr_val
                                    attr_val = attr_val.trim_matches(' ');

                                    // Strip out quotes (", ') if attr_val is delimited by any
                                    if let Some(val) = attr_val.split(['"', '\''].as_ref()).nth(1) {
                                        attr_val = val;
                                    }

                                    if attr_val != val {
                                        return false
                                    }
                                }
                                None => return false
                            }
                        }
                        // Check if attr exists at all (with any value) in node
                        None => if node.attributes().get(attr) == None {
                            return false
                        }
                    }
                None => {
                    eprintln!("Did not find closing square-bracket (]) for selector {}... aborting.", obj);
                    return false
                }
            }
        _ => panic!("Only valid characters are: . # [ ... This isn't supposed to happen")
    }

    true
}


// A node that can be parsed by the parser. Must have tag and attributes
pub trait ParseNode {
    fn new() -> Self;

    fn tag(&self) -> &str;
    fn tag_mut(&mut self) -> &mut String;
    fn set_tag(&mut self, val: String);
    fn attributes(&self) -> &HashMap<String, String>;
    fn add_attr(&mut self, name: String, val: String);

    fn class_list(&self) -> Vec<&str> {
        match self.attributes().get("class") {
            // Classes are separated by space
            Some(list) => list.split(' ').collect(),
            None => Vec::new()
        }
    }
}

// impl<Node> Display for Node
// where Node: ParseNode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "<\x1b[92m{}\x1b[0m \x1b[36m{:?}\x1b[0m>", self.tag(), self.attributes())
//     }
// }

#[derive(Debug)]
pub struct ParsedNode {
    pub tag: String,
    pub attributes: HashMap<String, String>
}
impl ParseNode for ParsedNode {
    fn new() -> Self {
        Self{ tag: String::new(), attributes: HashMap::new() }
    }

    fn tag(&self) -> &str {
        self.tag.as_str()
    }
    fn tag_mut(&mut self) -> &mut String {
        &mut self.tag
    }
    fn set_tag(&mut self, val: String) {
        self.tag = val;
    }
    fn attributes(&self) -> &HashMap<String, String> {
        &self.attributes
    }
    fn add_attr(&mut self, name: String, val: String) {
        self.attributes.insert(name, val);
    }
}
impl Display for ParsedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<\x1b[92m{}\x1b[0m \x1b[36m{:?}\x1b[0m>", self.tag(), self.attributes())
    }
}