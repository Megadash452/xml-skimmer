# XML Skimmer | Written in Rust

Skim through an XML file and find nodes using CSS Selectors.

For now all it can do is parse through regular and self-closing nodes (not including commetns and other default self-closing nodes).

## Handle Nodes
The `skim_xml` function takes a set of handlers (closures) paired with a CSS selector string. The selectors will try to match nodes as they are being parsed, so when a node that matches one (or more) of the selectors it will call the handler paired with that selector.

## Performance
Tested the program running with the [benchmark](src/benchmark.xml) source file, which has 2000 lines, 1000 depth levels, 8 attributes on each level (where 2 of those attrbibutes are overriden). 

(using `time` command in Linux):
```
real    0m1.311s
user    0m0.435s
sys     0m0.151s
```

## TODO list:
Things that work:
 - [x] duplicate attributes (the value of the last one read will be the value of that attribute)
 - [x] attributes not separated by whitespace
 - [x] Boolean Attributes (e.g.: `<tag attr>`)
 - [x] Self-closing nodes (e.g.: `<tag/>`)
 - [x] Attributes with single quotes (e.g.: `<tag attr='val'>`)
 - [x] Using other quote type in attr value (e.g.: `<tag attr='val"'>`)
 - [x] Space between AttrName and = `<tag attr = "val"/>`
 - [x] Comments
 - [x] Prolog node (`<?xml version="1.0"?>`) (treated as comment)

Things that DON'T work:
 - [ ] Cdata
 - [ ] namespaces
 - [ ] Text nodes

Tested Scenarios:
```xml
<?xml version="1.0" encoding="UTF-8"?>
<tag attr="val"></tag>
<tag attr="val" attr="val2"></tag>
<tag attr="val"attr2="val2"></tag>
<tag attr attr2></tag>
<tag/>
<tag attr/>
<tag attr="val"/>
<tag attr='val'/>
<tag attr="va'l"/>
<tag attr='va"l'/>
<tag attr= "val"/>
<tag attr ="val"/>
<tag attr = "val"/>
<!--comment-->
```
