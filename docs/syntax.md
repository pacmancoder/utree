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

### Bindings
Node attribute values and text content nodes can use bindings to watch changes of specific property.
For this, special syntax @ident can be used in place of content which we are trying to bind, e.g.:
`div>{@content}`, `p[class=@pclass]`, `(li>{@collection%text}) * @collection`

### Node duplication and collection mapping
Nodes could be duplicated or be bound to the collection via `*<number|@collection[%sub%path]>` operator.

E.g. `(ul>li*3) + ol>(li>b>{hello})*2` will produce the following code when rendered to html:
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

#### Collections
Let's say we have collection `items` in out component. each `items` element have `name`
and `id` properties. then, to create list based on this colection we can use the following syntax:
`ul>(li[id=@items%id]>div.list_icon+{My name is @items%name}) * @items`
which with items == `[{name=one id=item1}, {name=two id=item2}, {name=three id=item3}]` will produce:
```html
<ul>
    <li id="item1"><div class="list_icon"></div>My name is one</li>
    <li id="item2"><div class="list_icon"></div>My name is two</li>
    <li id="item3"><div class="list_icon"></div>My name is three</li>
</ul>
```

### Nested components
Components can be nested with `$component_name` binding expressions:
Let's say ve have parenc component with code `html>body>$body` and mody component with code `div>p>{hello}`
Then resulting component will be:
```html
<html>
    <body>
        <div>
            <p>hello</p>
        </div>
    </body>
</html>
```

This also can be used for collections of components.
e.g.: `ul>$items * @items` where `@items` has 2 components of `li>{hi}` will produce:
```html
<ul>
    <li>hi</li>
    <li>hi</li>
</ul>
```

Component binding also could make use of binding subpath expression to access nested components
