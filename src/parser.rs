use pest_derive::Parser;


#[derive(Parser)]
#[grammar = "syntax.pest"]
pub struct SyntaxParser;


#[cfg(test)]
mod tests
{
    use super::*;
    use expect_test::{expect, Expect};
    use pest::{Parser, iterators::Pair};

    fn parse_input(input: &str, rule: Rule) -> String {
        match SyntaxParser::parse(rule, input) {
            Ok(pairs) => {
                let lines: Vec<_> = pairs.map(|pair| {
                    format_pair(pair, 0, true)
                }).collect();
                let lines = lines.join("\n");
                format!("{}", lines)
            }
            Err(error) => format!("{}", error),
        }
    }

    // "Borrowed" from the pest web site :)
    fn format_pair(pair: Pair<Rule>, indent_level: usize, is_newline: bool) -> String {
        let indent = if is_newline {
            "  ".repeat(indent_level)
        } else {
            "".to_string()
        };

        let children: Vec<_> = pair.clone().into_inner().collect();
        let len = children.len();
        let children: Vec<_> = children.into_iter().map(|pair| {
            format_pair(pair, if len > 1 { indent_level + 1 } else { indent_level }, len > 1)
        }).collect();

        let dash = if is_newline {
            "- "
        } else {
            ""
        };

        match len {
            0 => format!("{}{}{:?}: {:?}", indent, dash, pair.as_rule(), pair.as_span().as_str()),
            1 => format!("{}{}{:?} > {}", indent, dash, pair.as_rule(), children[0]),
            _ => format!("{}{}{:?}\n{}", indent, dash, pair.as_rule(), children.join("\n"))
        }
    }


    fn assert_parsed(rule: Rule, s: &str, expect: Expect) {
        expect.assert_eq(&parse_input(s, rule));
    }

    #[test]
    fn identifier() {
        assert_parsed(
            Rule::ident,
            "_test-ABC_01 //comment",
            expect![[r#"- ident: "_test-ABC_01""#]]
        );

        SyntaxParser::parse(Rule::ident, "0test").expect_err("Identifiers starting with digit are not allowed");
    }

    #[test]
    fn node() {
        assert_parsed(
            Rule::node,
            "hello#my-id.class1.class2",
            expect![[r#"
                - node
                  - node_name: "hello"
                  - id_prop: "my-id"
                  - class_prop: "class1"
                  - class_prop: "class2""#]]
        );

        SyntaxParser::parse(Rule::ident, "#myid").expect_err("Standalone id is undefined");
        SyntaxParser::parse(Rule::ident, ".class").expect_err("Standalone class is undefined");
    }

    #[test]
    fn attrs() {
        assert_parsed(
            Rule::node,
            "hello[attr1=value attr2 = \" value \"]#id.class1.class2",
            expect![[r#"
                - node
                  - node_name: "hello"
                  - attrs_prop
                    - attr
                      - attr_name: "attr1"
                      - attr_value > ident: "value"
                    - attr
                      - attr_name: "attr2"
                      - attr_value > string > string_inner: " value "
                  - id_prop: "id"
                  - class_prop: "class1"
                  - class_prop: "class2""#]]
        );
    }

    #[test]
    fn content() {
        assert_parsed(
            Rule::tree,
            "(hello[attr1=value]>{hello}) + {@test wow 123 'hi'}",
            expect![[r#"
                - term
                  - term > node
                    - node_name: "hello"
                    - attrs_prop > attr
                      - attr_name: "attr1"
                      - attr_value > ident: "value"
                  - child_op: ">"
                  - term > text_node > ident: "hello"
                - sibling_op: "+"
                - term > text_node
                  - binding > binding_part: "test"
                  - ident: "wow"
                  - number: "123"
                  - string > string_inner: "hi"
                - EOI: """#]]
        );
    }

    #[test]
    fn groups() {
        assert_parsed(
            Rule::tree,
            "html>(body+head>div+(a>b)+p>a)\n+footer",
            expect![[r#"
                - term > node > node_name: "html"
                - child_op: ">"
                - term
                  - term > node > node_name: "body"
                  - sibling_op: "+"
                  - term > node > node_name: "head"
                  - child_op: ">"
                  - term > node > node_name: "div"
                  - sibling_op: "+"
                  - term
                    - term > node > node_name: "a"
                    - child_op: ">"
                    - term > node > node_name: "b"
                  - sibling_op: "+"
                  - term > node > node_name: "p"
                  - child_op: ">"
                  - term > node > node_name: "a"
                - sibling_op: "+"
                - term > node > node_name: "footer"
                - EOI: """#]]
        );
    }

    #[test]
    fn mul() {
        assert_parsed(
            Rule::tree,
            "html*3+(a>b)*@collection",
            expect![[r#"
                - term_list
                  - node > node_name: "html"
                  - multiplier > number: "3"
                - sibling_op: "+"
                - term_list
                  - term > node > node_name: "a"
                  - child_op: ">"
                  - term > node > node_name: "b"
                  - multiplier > binding > binding_part: "collection"
                - EOI: """#]]
        );
    }

    #[test]
    fn complex_binding() {
        assert_parsed(
            Rule::tree,
            "html[ id=@collection%id ] * @collection",
            expect![[r#"
                - term_list
                  - node
                    - node_name: "html"
                    - attrs_prop > attr
                      - attr_name: "id"
                      - attr_value > binding
                        - binding_part: "collection"
                        - binding_part: "id"
                  - multiplier > binding > binding_part: "collection"
                - EOI: """#]]
        );
    }

    #[test]
    fn component_binding() {
        assert_parsed(
            Rule::tree,
            "$html + div",
            expect![[r#"
                - term > node_binding > binding_part: "html"
                - sibling_op: "+"
                - term > node > node_name: "div"
                - EOI: """#]]
        );
        assert_parsed(
            Rule::tree,
            "($data%view + p>{test})*@data",
            expect![[r#"
                - term_list
                  - term > node_binding
                    - binding_part: "data"
                    - binding_part: "view"
                  - sibling_op: "+"
                  - term > node > node_name: "p"
                  - child_op: ">"
                  - term > text_node > ident: "test"
                  - multiplier > binding > binding_part: "data"
                - EOI: """#]]
        );
    }

    #[test]
    fn smoke_test() {
        let input = r#"
            html
                >(head
                    >meta[charset="utf-8"]
                    +(title>{"My page!"})
                    +link[
                        rel=stylesheet
                        href=@css_styles
                        ]
                )
                + body>div#page-body>div#list
                    >$items%view * @items
        "#;
        assert_parsed(Rule::tree, input, expect![[r#"
            - term > node > node_name: "html"
            - child_op: ">"
            - term
              - term > node > node_name: "head"
              - child_op: ">"
              - term > node
                - node_name: "meta"
                - attrs_prop > attr
                  - attr_name: "charset"
                  - attr_value > string > string_inner: "utf-8"
              - sibling_op: "+"
              - term
                - term > node > node_name: "title"
                - child_op: ">"
                - term > text_node > string > string_inner: "My page!"
              - sibling_op: "+"
              - term > node
                - node_name: "link"
                - attrs_prop
                  - attr
                    - attr_name: "rel"
                    - attr_value > ident: "stylesheet"
                  - attr
                    - attr_name: "href"
                    - attr_value > binding > binding_part: "css_styles"
            - sibling_op: "+"
            - term > node > node_name: "body"
            - child_op: ">"
            - term > node
              - node_name: "div"
              - id_prop: "page-body"
            - child_op: ">"
            - term > node
              - node_name: "div"
              - id_prop: "list"
            - child_op: ">"
            - term_list
              - node_binding
                - binding_part: "items"
                - binding_part: "view"
              - multiplier > binding > binding_part: "items"
            - EOI: """#]]);
    }
}
