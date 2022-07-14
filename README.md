# XML Parser | Written in Rust

For now all it can do is parse through regular and self-closing nodes (not including commetns and other default self-closing nodes).

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

Tested Scenarios:
```xml
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

Things that DON'T work:
 - [ ] Default self-closing nodes (e.g.: `<?xml?>`)
 - [ ] Cdata
 - [ ] namespaces
 - [ ] Text nodes


Things not tested:
 - how many nodes can the stack hold
 - nodes with empty tags