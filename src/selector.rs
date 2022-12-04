use std::{str::FromStr, collections::{HashMap, HashSet}};
use crate::ParsedNode;


/// Parses a string where a type that can be parsed is separated by commas.
/// Ignores commas inside **strings** (delimited by single `'` or double `"` quotes).
/// Also accepts 1 end trailing comma.
/// 
/// When matching with a [`ParsedNode`], if any of the inner selectors match the node,
/// then [`CommaSeparated::match_node()`] returns `true`.
/// 
/// ## Example
/// 
/// ```
/// use std::collections::HashMap;
/// use xml_skimmer::selector::{CommaSeparated, Selector};
/// 
/// assert_eq!("tag , tag2".parse::<CommaSeparated<Selector>>(), Ok(CommaSeparated(vec![
///     Selector { tag: "tag".to_string().into(), .. Default::default() },
///     Selector { tag: "tag2".to_string().into(), .. Default::default() },
/// ])));
/// 
/// assert_eq!("tag , tag2 , ".parse::<CommaSeparated<Selector>>(), Ok(CommaSeparated(vec![
///     Selector { tag: "tag".to_string().into(), .. Default::default() },
///     Selector { tag: "tag2".to_string().into(), .. Default::default() },
/// ])));
/// 
/// assert_eq!("tag[attr='1, 2, 3']".parse::<CommaSeparated<Selector>>(), Ok(CommaSeparated(vec![
///     Selector {
///         tag: "tag".to_string().into(),
///         attributes: HashMap::from([("attr".to_string(), "1, 2, 3".to_string().into())]),
///         .. Default::default() },
/// ])));
/// ```
#[derive(Debug, PartialEq)]
pub struct CommaSeparated<T: FromStr>(pub Vec<T>);
impl CommaSeparated<Selector> {
    pub fn match_node(&self, stack: &[ParsedNode]) -> bool {
        for selector in &self.0 {
            if selector.match_node(stack) {
                return true
            }
        }

        false
    }
}
impl<T: FromStr> FromStr for CommaSeparated<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rtrn = vec![];

        let mut start = 0;
        let mut i = 0;
        let mut string_quote: Option<char> = None;

        for c in s.chars() {
            i += 1;

            match (c, string_quote) {
                // Open string with single or double quotes
                ('\'' | '"', None) => string_quote = Some(c),
                // String opened with single or double quotes, and it closes with that same quote
                ('\'', Some('\'')) | ('"', Some('"')) => string_quote = None,
                // Found a comma, not in string
                (',', None) => {
                    // subtract i - 1 to exclude the comma
                    rtrn.push(T::from_str(&s[start..(i - 1)].trim())?);
                    start = i;
                },
                _ => {}
            }
        }

        // See if there is a T after the last comma
        let s = s[start..i].trim();
        if !s.is_empty() {
            rtrn.push(T::from_str(s)?);
        }

        Ok(Self(rtrn))
    }
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
/// # Combinators
/// 
/// The **parent** is the previous selector in the '[`Combinator`] stack'.
/// For example, with selector `parent > child`, the base selector has **tag** `"child"`,
/// and it has a **parent** with [`Child`](Combinator::Child) (`>`) with **tag** `"parent"`.
/// 
/// See [`Combinator`].
/// 
/// # [`FromStr`]
/// 
/// Can be parsed from a String (`"...".parse::<Selector>()` or `Selector::from_str("...")`).
/// String can contain *leading* and *trailing* `whitespace`.
/// 
/// The tag always goes first in the string.
/// There must only be 1 **tag** and **id** in the string.
/// 
/// **classes** is a [`HashSet`] and `attributes` is a [`HashMap`],
/// so a class or attribute name must not be found in the string more than once.
/// **attributes** can have no value `[attr]`, or Some value `[attr=val]`,
/// where the value can be wrapped in single `'` or double `"` quotes.
/// 
/// See [`SelectorParseError`] for possible errors when parsing from a string.
#[derive(Debug, Default, PartialEq)]
pub struct Selector {
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: HashSet<String>,
    pub attributes: HashMap<String, Option<String>>,
    pub parent: Option<(Box<Selector>, Combinator)>
}
impl Selector {
    pub fn match_node(&self, stack: &[ParsedNode]) -> bool {
        let mut node_iter = stack.iter().rev();
        let mut sel_iter = Some(self);
        let mut combinator = None;

        while let Some(selector) = sel_iter {
            match combinator {
                // Some ancestor node in the stack has to match
                Some(Combinator::Descendant) => {
                    let mut matched = false;
                    // Try again for every node until one matches
                    while let Some(node) = node_iter.next() {
                        if selector.match_simple(node) {
                            matched = true;
                            break
                        }
                    }

                    // No node in the stack matched
                    if !matched {
                        return false
                    }
                },
                // The directly next node in the stack has to match.
                // This also happens with the first selector: e.g. "... tag".
                Some(Combinator::Child) | None =>
                    match node_iter.next() {
                        Some(node) =>
                            if !selector.match_simple(node) {
                                return false
                            },
                        // stack was empty
                        None => return false
                    }
            }

            sel_iter = match &selector.parent {
                Some((parent, comb)) => {
                    combinator = Some(*comb);
                    Some(&*parent)
                },
                // finish
                None => None
            }
        }

        true
    }

    /// Match a single selector without considering combinators.
    fn match_simple(&self, node: &ParsedNode) -> bool {
        if let Some(ref tag) = self.tag {
            if node.tag != *tag {
                return false
            }
        }
        
        match (node.attributes.get("id"), &self.id) {
            // Both node and selector have an id to match
            (Some(node_id), Some(id)) =>
                if *node_id != *id {
                    return false
                },
            // Node doesn't have id
            (None, Some(_)) => return false,
            _ => {}
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
        s = s.trim_start();

        let mut current_sel = Self::default();

        let mut chars = s.chars();
        let mut push_to = PushTo::Tag;
        let mut buf = String::new();

        /// Assign a string to whatever part of the selector it needs to.
        /// 
        /// Will return [`Err`] when trying to PushTo [`Id`](PushTo::Id) or [`Classes`](PushTo::Classes) and **buf** is empty.
        /// This happens when `#` or `.` are the last char of the string.
        fn push(to: PushTo, sel: &mut Selector, buf: String) -> Result<(), SelectorParseError> {
            match to {
                // When push_to is `Tag` and *buf* is empty, push_to should be trated as `None`.
                PushTo::Tag if buf.is_empty() => {},

                PushTo::Tag =>
                    match sel.tag {
                        Some(_) => return Err(SelectorParseError::MultipleTags),
                        None => sel.tag = Some(buf)
                    },
                PushTo::Id =>
                    match sel.id {
                        Some(_) => return Err(SelectorParseError::MultipleIDs),
                        None if !buf.is_empty() => sel.id = Some(buf),
                        None => return Err(SelectorParseError::EmptyToken)
                    },
                PushTo::Classes =>
                    if !buf.is_empty() {
                        sel.classes.insert(buf);
                    } else {
                        return Err(SelectorParseError::EmptyToken)
                    },
                // When one of these chars is in the attribute: [at#tr] or [at.tr] or [at[tr] 
                PushTo::AttrName => return Err(SelectorParseError::UnclosedBracket)
            }
            Ok(())
        }

        while let Some(character) = chars.next() {
            match character {
                '#' | '.' | '[' => {
                    // buf could be empty if its the first char in s, or right after a `]`.
                    push(push_to, &mut current_sel, buf)?;
                    // Reset buffers
                    buf = String::new();
                    push_to = PushTo::new(character);
                },
                '=' => match push_to {
                    PushTo::AttrName => {
                        if buf.is_empty() {
                            return Err(Self::Err::EmptyToken)
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
                            Some(']' | '=') => return Err(Self::Err::BadChar),

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

                        current_sel.attributes.insert(buf, Some(val_buf));

                        // reset buffers
                        buf = String::new();
                        push_to = PushTo::Tag;
                    },
                    _ => return Err(Self::Err::BadChar)
                },
                // When attr has no value: [attr]
                ']' => match push_to {
                    PushTo::AttrName => {
                        if buf.is_empty() {
                            return Err(Self::Err::EmptyToken)
                        }
                        current_sel.attributes.insert(buf, None);

                        // Reset buffers
                        buf = String::new();
                        push_to = PushTo::Tag;
                    },
                    _ => return Err(Self::Err::BadChar/*(c)*/)
                },
                // Whitespace inside attributes is ignored,
                // otherwise, it means the next tokens will go to a child selector 
                _ if character.is_whitespace() =>
                    if push_to != PushTo::AttrName {
                        // buf could be empty if its the first char in s, or right after a `]`.
                        push(push_to, &mut current_sel, buf)?;
                        // reset buffer
                        buf = String::new();

                        // the first char of the next child selector
                        let mut first_c = None;
                        let mut combinator = Combinator::Descendant;
                        // Look for the combinator within the whitespace.
                        // If there is only whitespace, combinator is Descendant.
                        // Also parse through the trailing whitespace of the combinator.
                        while let Some(c) = chars.next() {
                            match c {
                                '>' => // Check if a combinator was already found
                                    if combinator == Combinator::Descendant {
                                        combinator = Combinator::Child;
                                    } else {
                                        // When have this situation: "tag > >..."
                                        // Combinators cannot be used as prefixes.
                                        return Err(SelectorParseError::UnknownPrefix)
                                    },
                                _ if c.is_whitespace() => {},
                                _ => {
                                    first_c = Some(c);
                                    break
                                }
                            }
                        }

                        let c = match first_c {
                            Some(c) => c,
                            // Selector ends with trailing whitespace
                            None if combinator == Combinator::Descendant => return Ok(current_sel),
                            None => return Err(Self::Err::NoOtherSideCombinator)
                        };
                        push_to = PushTo::new(c);
                        if push_to == PushTo::Tag {
                            buf.push(c)
                        }

                        // Set current selector to parent of a new selector.
                        current_sel = Self {
                            parent: Some((Box::new(current_sel), combinator)),
                            ..Default::default()
                        }
                    },
                // Any punct char (except `-` and `_`) is considered a prefix or combinator
                _ if character.is_ascii_punctuation()
                    && character != '-'
                    && character != '_' => return Err(Self::Err::UnknownPrefix),
                _ => buf.push(character)
            }
        }

        // When reached the end of the string, push what is in the buffer
        push(push_to, &mut current_sel, buf)?;
        
        Ok(current_sel)
    }
}


#[derive(Debug, PartialEq)]
pub enum SelectorParseError {
    MultipleTags,
    MultipleIDs,
    /// When the last char of the string was `#`, or `.`,
    /// Or when have empty brackets: `[]`. Therefore,
    /// this happens when trying to create an **Id**, **Class**, or **Attribute**
    /// but their strings would be empty.
    EmptyToken,
    /// When there is a punctuation that is not
    /// `#`, `.`, or `[` in the selector string.
    UnknownPrefix,
    UnclosedString,
    UnclosedBracket,
    /// When found a combinator, but there is no selector after it.
    NoOtherSideCombinator,
    /// A [`char`] was found in a position
    /// that it wasn't supposed to be in.
    BadChar,
    WhiteSpace,
    EmptyString,
}

/// Separates [`Selector`]s to match [`Node`](ParsedNode)s in different ways.
/// `SelectorA <Combinator> SelectorB`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Combinator {
    /// Is denoted by `>`.
    /// The selector will only match nodes `B`
    /// that are direct children of a node that matches `A`.
    Child,
    /// Is denoted by `whitespace`.
    /// The selector nodes `B` if one of its ancestors matches `A`.
    Descendant
}


#[derive(PartialEq)]
enum PushTo {
    Tag, Id, Classes, AttrName
}
impl PushTo {
    pub fn new(c: char) -> Self {
        match c {
            '#' => Self::Id,
            '.' => Self::Classes,
            '[' => Self::AttrName,
            _ => Self::Tag
        }
    }
}
