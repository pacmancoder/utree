# μTree syntax

μTree syntax is pretty similar to [emmet abbreviations](https://docs.emmet.io/abbreviations/syntax/), but it provides more features for better flexibility and readability

### Reference

#### Identifiers
Identifiers are used to declare tree node names, attributes or uTree variables.
The following characters can be used in the identifiers:: `a-z`, `A-Z`, `0-9`, `_`, `-`. However, for the first character, `0-9`, `$` and `-` can't be used. If these characters are needed to be used as a first character, for example in attribute value, use string representation instead (e.g. `ident_representation="007string_representation"`).

#### Tree navigation
`>` - moves down through hierarchy, effectively sets last declared element the as current active element. Note that if tried to use on multiple elements instead of one, expression will fail:
- `div*5>p` - ERROR
- `(div>p)*5` - OK
- `div>p+b>i+a` - OK

`+` - create next element as a sibbling of active element. Not that this operator does not change currently active element.

E.g. when using `div>p+(a>b)+a>i` uTree expression for rendering HTML document, the following code will be generated:
```html
<div>
    <p></p>
    <a>
        <b></b>
    </a>
    <a>
        <i></i>
    </a>
</div>
```

### Node inner text
To create a text node, `{identifier}` | `{"string with whitespaces"}` | `{123}`
syntax can be used.

E.g. `p>{text1}+b>{"text 2"}` will generate following when rendered to html:
```html
<p>
    text1
    <b>text 2</b>
</p>
```

### Attributes
Attributes can be set via note attributes syntax: `node_name.class1.class2#id1[attribute1_name="value" attr=42 attr=ident_like_value]`. Attribute names reuired to follow identifier rules, while attribute values could use either identifier, string or number representation.

- `.my-class` is a shorthand for `[class="my-class"]`
- `#my-id` is a shorthand for `[id="my-id"]`

E.g. `div#button1.btn.alert[onclick="press_callback()"]>b>{text}` will produce the following if rendered to html:
```html
    <div id="button1" class="btn alert" onclick="press_callback()">
        <b>text</b>
    </div>
```

### Node duplication
Nodes could be duplicated via `*<number>` operator. Also, can be applied to element groups

E.g. `ul>li*3^ol>(li>b"hello")*2` will produce the following code when rendered to html:
```html
<ul>
    <li></li>
    <li></li>
    <li></li>
</ul>
<ol>
    <li><b>hello</b></li>
    <li><b>hello</b></li>
</ol>
```

### Numeration
Identifiers or string values could be dynamically generated when duplicating elements - any occurence of `$` character inside indents will be replaced with generated number when `*` operatir is used for node duplication:

E.g. `ul>li#item-$"Item $ content"*5` will produce the following when rendered to html:
```html
<ul>
    <li id="item-1">Item 1 content</li>
    <li id="item-2">Item 2 content</li>
    <li id="item-3">Item 3 content</li>
    <li id="item-4">Item 4 content</li>
    <li id="item-5">Item 5 content</li>
</ul>
```

Also, variables could be used to make substitutions even more flexible!

E.g. `(ul#list-$[y]>li#item-$[y]-$[x]{Item $[x]}*3x)*2y` will generate the following when rendered to html:
```html
<ul id="list-1">
    <li id="item-1-1">Item 1</li>
    <li id="item-1-2">Item 2</li>
    <li id="item-1-3">Item 3</li>
</ul>
<ul id="list-2">
    <li id="item-2-1">Item 1</li>
    <li id="item-2-2">Item 2</li>
    <li id="item-2-3">Item 3</li>
</ul>
```

`$` sibstitution character can be escaped in identifiers via `$$` and via `\$` in strings

Numbering order can be modified via `&<modifier>` operator, where `<modifier>` is one of pre-defined modifiers or custom-registered modifiers

##### Pre-defined modifiers
- `rev` - reverses numbering sequence
- `from:*` - counts from value specified by wildcard instead of 1
- `inc:*` - counts in increments specified by wildcard instead of 1

E.g `li#item-$[x]*5x&from-0&inc-2&rev` will generate the following when rendered to html:
```html
<li id="item-8"></li>
<li id="item-6"></li>
<li id="item-4"></li>
<li id="item-2"></li>
<li id="item-0"></li>
```

##### Element instantiation callbacks

We can set "hooks" for element instantiation in tree via `@<key>` operator. uTree engine will call `on_rendered(key: String, element: TreeElement<T>)` when the tree element gets instantiated. `<key>` here could be any identifier, and even could contain variables when used with node duplication: `li@item-$*3&rev`:

```html
<li></li>
<li></li>
<li></li>
```
```
Callback calls:
- on_rendered for "item-1"
- on_rendered for "item-2"
- on_rendered for "item-3"
```

This could be useful when building DOM trees in wasm to acquire generated DOM elements without need to do expensive element by id querying for each element manually
