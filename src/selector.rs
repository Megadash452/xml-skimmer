use std::{str::FromStr, collections::{HashMap, HashSet}};
use crate::ParsedNode;


/// Parses a string where a type that can be parsed is separated by commas.
/// Also accepts 1 end trailing comma.
/// 
/// ## Example
/// 
/// ```
/// use xml_skimmer::selector::{comma_separated, Selector};
/// 
/// assert_eq!(comma_separated::<Selector>("tag , tag2 , "), Ok(vec![
///     Selector { tag: "tag".to_string().into(), .. Default::default() },
///     Selector { tag: "tag2".to_string().into(), .. Default::default() },
/// ]));
/// ```
pub fn comma_separated<T: FromStr>(s: &str) -> Result<Vec<T>, T::Err> {
    let mut rtrn = vec![];
    let mut splits = s.split(',');

    while let Some(mut s) = splits.next() {
        s = s.trim();
        // is not a trailing comma
        if splits.clone().next() != None || !s.is_empty() {
            rtrn.push(T::from_str(s.trim())?);
        }
    }

    Ok(rtrn)
}


/// A CSS selector that can be matched against an XML node.
/// 
/// Supported tokens are: `tag`, `#id`, `.class`, `[attr]`,
/// `[attr=val]`, `[attr="val"]` (single or double quotes).
/// 
/// When an **attribute** in the selector has no value (`[attr]`),
/// it means that when matching whith an XML node
/// it will only check if the attribute exists at all with any value.
/// But when an **attribute** in the selector has an empty value (`[attr=""]`),
/// it will check if it has that attribute,
/// and also requires that the XML node specifically has an empty string as the value for that attribute.
/// 
/// # [`FromStr`]
/// 
/// Can be parsed from a String (`"".parse::<Selector>()` or `Selector::from_str("")`).
/// Whitespace is not allowed in parsing (except inside attribute brackets `[]`)
/// because it is used for node hierarchy, which this struct does not support // TODO: (yet, also '*' and '>' selector operators).
/// 
/// The tag always goes first in the string.
/// 
/// `classes` is a [`HashSet`] and `attributes` is a [`HashMap`],
/// so a class or attribute name must not be found in the string more than once.
/// Attributes can have no value `[attr]`, or Some value `[attr=val]`,
/// where the value can be wrapped in single `'` or double `"` quotes.
/// 
/// There must only be 1 **id** in the string.
/// 
/// See [`SelectorParseError`] for possible errors when parsing from a string.
#[derive(Debug, Default, PartialEq)]
pub struct Selector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: HashSet<String>,
    pub attributes: HashMap<String, Option<String>>
}
impl Selector {
    pub fn match_node(&self, node: &ParsedNode) -> bool {
        if let Some(ref tag) = self.tag {
            if node.tag != *tag {
                return false
            }
        }

        if let Some(ref id) = self.id {
            match node.attributes.get("id") {
                Some(node_id) =>
                    if *node_id != *id {
                        return false
                    },
                None => return false
            }
        }
            
        let class_list = node.class_list();

        for class in self.classes.iter() {
            if !class_list.contains(class.as_str()) {
                return false
            }
        }

        for attr in self.attributes.iter() {
            match attr.1 {
                // [attr = val]
                Some(attr_val) => match node.attributes.get(attr.0) {
                    Some(node_attr_val) =>
                        if *node_attr_val != *attr_val {
                            return false
                        },
                    // Node does not have attribute
                    None => return false
                },
                // [attr]
                None =>
                    if !node.attributes.contains_key(attr.0) {
                        return false
                    }
            }
        }

        true
    }
}
impl FromStr for Selector {
    type Err = SelectorParseError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Self::Err::EmptyString)
        }

        let tag = {
            let (tag, rest) = split_tag(s)?;
            s = rest;

            if tag.is_empty() {
                None
            } else {
                Some(tag.to_string())
            }
        };

        let mut id: Option<String> = None;
        let mut classes = HashSet::new();
        let mut attributes = HashMap::new();

        let mut buf = String::new();
        let mut chars = s.chars();

        /// Whether the characters being parsed are for ID, a Class, or an Attribute
        #[derive(PartialEq)]
        enum PushTo {
            Id, Classes, AttrName
        }
        impl PushTo {
            pub fn new(c: char) -> Result<Self, SelectorParseError> {
                match c {
                    '#' => Ok(Self::Id),
                    '.' => Ok(Self::Classes),
                    '[' => Ok(Self::AttrName),
                    _ => Err(SelectorParseError::UnknownPrefix)
                }
            }
        }

        let mut push_to = PushTo::new(match chars.next() {
            Some(c) => c,
            None => return Ok(Self { tag, id, classes, attributes })
        })?;

        // Find class, id, other attributes
        while let Some(character) = chars.next() {
            match character {
                '#' | '.' | '[' => {
                    match push_to {
                        PushTo::Id => if id.is_some() {
                            return Err(Self::Err::MultipleIDs)
                        } else {
                            id = Some(buf)
                        },
                        PushTo::Classes => if !classes.insert(buf) {
                                // If the set already contained this class
                                return Err(Self::Err::DuplicateClass)
                            },
                        // When one of these chars is in the attribute: [at#tr] or [at.tr] or [at[tr] 
                        PushTo::AttrName => return Err(Self::Err::UnclosedBracket)
                    }

                    // reset buffers
                    buf = String::new();
                    push_to = PushTo::new(character)?;
                },
                '=' => match push_to {
                    PushTo::AttrName => {
                        if buf.is_empty() {
                            return Err(Self::Err::EmptyAttrName)
                        }
                        // skip whitespace before attribute
                        let mut next = None;
                        while let Some(character) = chars.next() {
                            if !character.is_whitespace() {
                                next = Some(character);
                                break
                            }
                        }

                        let mut val_buf = String::new();
                        let opening_quote = match next {
                            Some('"') => Some('"'),
                            Some('\'') => Some('\''),
                            // When there is nothing after EqSign '=': [attr=]
                            Some(']') => return Err(Self::Err::BadChar),

                            Some(character) => {
                                val_buf.push(character);
                                None
                            },
                            None => None
                        };

                        let mut found_closing_quote = false;
                        let mut found_closing_bracket = false;
                        // Find closing quote (if there was an opening quote)
                        if let Some(quote) = opening_quote {
                            while let Some(character) = chars.next() {
                                if character == quote {
                                    found_closing_quote = true;
                                    break
                                }
                                val_buf.push(character)
                            }
                            // also find ']'
                            while let Some(character) = chars.next() {
                                if character == ']' {
                                    found_closing_bracket = true;
                                    break
                                }
                                if !character.is_whitespace() {
                                    return Err(Self::Err::BadChar)
                                }
                            }
                        } else {
                            // The value is every character until ']' or whitespace
                            while let Some(character) = chars.next() {
                                if character.is_whitespace() {
                                    break
                                }
                                if character == ']' {
                                    found_closing_bracket = true;
                                    break
                                }
                                val_buf.push(character)
                            }
                            // also find ']'
                            if !found_closing_bracket {
                                while let Some(character) = chars.next() {
                                    if character == ']' {
                                        found_closing_bracket = true;
                                        break
                                    }
                                    if !character.is_whitespace() {
                                        return Err(Self::Err::BadChar)
                                    }
                                }
                            }
                        }

                        if opening_quote.is_some() && !found_closing_quote {
                            return Err(Self::Err::UnclosedString)
                        }
                        if !found_closing_bracket {
                            return Err(Self::Err::UnclosedBracket)
                        }

                        if let Some(_) = attributes.insert(buf, Some(val_buf)) {
                            // If this attribute already existed
                            return Err(Self::Err::DuplicateAttr)
                        }

                        // reset buffers
                        buf = String::new();
                        match chars.next() {
                            Some(c) => push_to = PushTo::new(c)?,
                            None => break
                        }
                    },
                    _ => return Err(Self::Err::BadChar)
                },
                ']' => match push_to {
                    PushTo::AttrName => {
                        if buf.is_empty() {
                            return Err(Self::Err::EmptyAttrName)
                        }
                        if let Some(_) = attributes.insert(buf, None) {
                            // If this attribute already existed
                            return Err(Self::Err::DuplicateAttr)
                        }

                        // reset buffers
                        buf = String::new();
                        match chars.next() {
                            Some(c) => push_to = PushTo::new(c)?,
                            None => break
                        }
                    },
                    _ => return Err(Self::Err::BadChar)
                },
                // whitespace is only allowed in attributes (and is ignored)
                _ if character.is_whitespace() =>
                    if push_to != PushTo::AttrName {
                        return Err(Self::Err::WhiteSpace)
                    },
                _ => buf.push(character)
            }
        }

        // When reached the end of the string, push what is in the buffer
        if !buf.is_empty() {
            match push_to {
                PushTo::Id => if id.is_some() {
                    return Err(Self::Err::MultipleIDs)
                } else {
                    id = Some(buf)
                },
                PushTo::Classes => if !classes.insert(buf) {
                    // If the set already contained this class
                    return Err(Self::Err::DuplicateClass)
                },
                // When one of these chars is in the attribute: [at#tr] or [at.tr] or [at[tr] 
                PushTo::AttrName => return Err(Self::Err::UnclosedBracket)
            }
        }

        Ok(Self { tag, id, classes, attributes })
    }
}


/// Get the slice that represents the tag in a css selector, and also the slice after.
/// - e.g.: `tag#id.cls[attr=val]` -> (`tag`, `#id.cls[attr=val]`)
/// 
/// The **tag** ends before a [`punctuation`] character. Whitespace is not allowed.
fn split_tag(selector: &str) -> Result<(&str, &str), SelectorParseError> {
    // The end index of the tag slice
    let mut end = 0;

    for character in selector.chars() {
        if character.is_ascii_punctuation() {
            return Ok((&selector[..end], &selector[end..]))
        }
        if character.is_whitespace() {
            return Err(SelectorParseError::WhiteSpace)
        }
        end += 1
    }

    // The entire selector is the tag
    return Ok((selector, ""))
}


#[derive(Debug, PartialEq)]
pub enum SelectorParseError {
    MultipleIDs,
    DuplicateClass, // ? might remove
    DuplicateAttr, // ? might remove
    EmptyAttrName,
    /// When there is a punctuation that is not
    /// `#`, `.`, or `[` in the selector string.
    UnknownPrefix,
    UnclosedString,
    UnclosedBracket,
    /// A [`char`] was found in a position
    /// that it wasn't supposed to be in.
    BadChar,
    WhiteSpace,
    EmptyString,
}
