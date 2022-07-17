use std::{collections::HashMap, fmt::Display};

/// Match a css selector to a node. Only supports `tag`, `#id`, `.class`, `[attr]`, `[attr="val"]`
/// - The `tag` should be the first part of the selector, and is optional.
/// - Then anything preceeded with `#` is an id. Looks for attribute `id` in node.
/// - Then anything preceeded with `.` is a class. Looks for attribute `class` in node.
/// - Then anything wrapped in `[]` is any other attribute.
///   - `[attr]` checks if an attribute exists at all in node.
///   - `[attr="val"]` or checks that an attribute has the specified value in node.
///     - Aliases: `[attr='val']`, `[attr=val]`
/// <hr><br>
/// Not allowed: empty selector, and whitespace (' ', \t, \n)
pub fn match_to_node(node: &ParsedNode, selector: &str) -> bool {
    if selector.is_empty() {
        eprintln!("Empty selectors are not valid");
        return false
    }

    let mut iter = selector.chars();
    // The selector tag is optional
    let mut tag: Option<&str> = None;
    // The start of a selector object (tag, class, id, attr)
    let mut start: usize = 0;

    // Find selector tag (should be the first thing in the selector)
    while let Some(character) = iter.next() {
        match character {
            ' ' | '\t' | '\n' => {
                eprintln!("No whitespace allowed in selectors");
                return false
            }
            '.' | '#' | '[' => {
                // Slice selector up to one of . # [ (indexed by start)
                tag = Some(&selector[0..start]);
                // When tag is a &str not empty, match selector tag with node tag
                if !tag.unwrap().is_empty() && tag.unwrap() != node.tag {
                    return false
                }
                break;
            }
            _ => {}
        }
        // Set up start so that it is at the beginning of a . # or [
        start += 1;
    }

    // selector doesnt have . # or [
    if tag == None {
        // The entire selector is the tag
        // Done with entire selector, no need to continue any further
        return selector == node.tag
    }

    // Get the classlist of the node
    let classlist: Vec<&str> = match node.attributes.get("class") {
        Some(list) => {
            // Classes are separated by space
            list.split(' ').collect()
        }
        None => Vec::new()
    };

    // The end of a selector object (tag, class, id, attr)
    let mut end: usize = start + 1;
    // Find class, id, other attributes
    while let Some(character) = iter.next() {
        match character {
            '.' | '#' | '[' => {
                // println!("obj: {:?}", &selector[start..end]);
                if !match_sel_obj_to_node(node, &selector[start..end], &classlist) {
                    return false
                }
                start = end;
            }
            _ => { }
        }
        end += 1;
    }

    // When reached the end of selector, do one more for the last obj
    match_sel_obj_to_node(node, &selector[start..end], &classlist)
}


/// Check if part of a css selector matches an attribute in node
/// - E.g: Check if `obj = ".cls"` matches in `Node{ _, attrs: {"class": "... cls ..."} }`
fn match_sel_obj_to_node(node: &ParsedNode, obj: &str, classlist: &Vec<&str>) -> bool{
    // First char of obj tells if it is class, id, or attr
    match obj.chars().nth(0) {
        // match selector classs with one of node classes
        Some('.') => {
            if !classlist.contains(&&obj[1..]) {
                return false
            }
        }
        // Match selector id with node id
        Some('#') => {
            // Check if node has attribute named "id"
            match node.attributes.get("id") {
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

                            // Check if attribute exists at all (with any value) in node
                            match node.attributes.get(attr_name) {
                                // Check if attribute exists with specific value in node
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
                        // Check if attribute exists at all (with any value) in node
                        None => if node.attributes.get(attr) == None {
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


#[derive(Debug)]
pub struct ParsedNode {
    pub tag: String,
    pub attributes: HashMap<String, String>
}
impl ParsedNode {
    pub fn new() -> Self {
        Self{ tag: String::new(), attributes: HashMap::new() }
    }
}
impl Display for ParsedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<\x1b[92m{}\x1b[0m \x1b[36m{:?}\x1b[0m>", self.tag, self.attributes)
    }
}