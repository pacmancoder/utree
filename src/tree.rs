use thiserror::Error;
use pest::iterators::{Pairs, Pair};
use crate::parser::Rule;
use std::fmt::Display;


#[derive(Debug, Clone)]
pub enum PropertyBinding {
    RootIdentifier(String),
    NestedIdentifier(Vec<String>),
}

impl Display for PropertyBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropertyBinding::RootIdentifier(ident) => write!(f, "@{}", ident),
            PropertyBinding::NestedIdentifier(idents) => write!(f, "@{}", idents.join("%")),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GenericValue {
    Text(String),
    Number(isize),
    Binding(PropertyBinding),
}

impl Display for GenericValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericValue::Text(s) => write!(f, "\"{}\"", s),
            GenericValue::Number(n) => write!(f, "{}", n),
            GenericValue::Binding(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    None,
    Single(GenericValue),
    Multiple(Vec<GenericValue>),
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeValue::None => f.write_str("<NONE>"),
            AttributeValue::Single(val) => write!(f, "{}", val),
            AttributeValue::Multiple(values) => write!(
                f,
                "{{{}}}",
                values
                    .iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

impl AttributeValue {
    pub fn append(&mut self, value: GenericValue) {
        *self = match std::mem::replace(self, Self::None) {
            AttributeValue::None => AttributeValue::Single(value),
            AttributeValue::Single(first) => AttributeValue::Multiple(vec![first, value]),
            AttributeValue::Multiple(mut list) => {
                list.push(value);
                AttributeValue::Multiple(list)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

impl Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

#[derive(Debug, Error)]
pub enum TreeBuildError {
    #[error("Leaf node can't have any children")]
    LeafNodeCantHaveChildren,
    #[error("Invalid number")]
    InvalidNumLiteral,
}

#[derive(Debug, Clone)]
pub struct RootTreeNode {
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub struct NormalTreeNode {
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<TreeNode>,
}

impl NormalTreeNode {
    pub fn append_attribute(&mut self, name: &str, value: GenericValue) {
        if let Some(existing) = self.attributes.iter_mut().find(|a| a.name == name) {
            existing.value.append(value);
        } else {
            self.attributes.push(Attribute {
                name: name.to_owned(),
                value: AttributeValue::Single(value)
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct InnerContentTreeNode {
    pub value: GenericValue,
}

#[derive(Debug, Clone)]
pub struct SubtreeTreeNode {
    pub property: PropertyBinding,
}

#[derive(Debug, Clone)]
pub struct CollectionTreeNode {
    pub nodes: Vec<TreeNode>,
    pub collection: PropertyBinding,
}

#[derive(Debug, Clone, from_variants::FromVariants)]
pub enum TreeNode {
    // Pseudo-node to hold all tree
    Root(RootTreeNode),
    // Normal tree node
    Normal(NormalTreeNode),
    // Represents inner node content
    InnerContent(InnerContentTreeNode),
    // Subtree bound to the given property
    Subtree(SubtreeTreeNode),
    // Special kind of tree node depresenting highly coupled nodes list (usually, created when bound to collection property)
    Collection(CollectionTreeNode)
}

impl Display for TreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        self.as_string_impl(&mut result, "");
        f.write_str(&result)
    }
}

impl TreeNode {
    pub fn children_mut(&mut self) -> Result<&mut Vec<TreeNode>, TreeBuildError> {
        match self {
            TreeNode::Root(n) => Ok(&mut n.children),
            TreeNode::Normal(n) => Ok(&mut n.children),
            _ => Err(TreeBuildError::LeafNodeCantHaveChildren),
        }
    }

    fn as_string_impl(&self, result: &mut String, ident: &str) {
        match self {
            TreeNode::Root(RootTreeNode { children }) => {
                // Do not display root node
                for child in children {
                    child.as_string_impl(result, "");
                }
            },
            TreeNode::Normal(NormalTreeNode { name, attributes, children }) => {
                let mut node = format!("{} - {}", ident, name.clone());
                if !attributes.is_empty() {
                    let attrs = attributes
                        .iter()
                        .map(|a| format!("{}={}", a.name, a.value))
                        .collect::<Vec<String>>().join(" ");
                    node.push_str(&format!("[{}]", attrs))
                }
                node.push('\n');
                result.push_str(&node);
                for child in children {
                    child.as_string_impl(result, &format!("{}  ", ident));
                }
            },
            TreeNode::InnerContent(InnerContentTreeNode { value }) => {
                result.push_str(&format!("{} - [CONTENT] {}\n", ident, value));
            },
            TreeNode::Subtree(SubtreeTreeNode { property }) => {
                result.push_str(&format!("{} - [SUBTREE] {}\n", ident, property));
            },
            TreeNode::Collection(CollectionTreeNode { nodes, collection }) => {
                result.push_str(&format!("{} - [COLLECTION] {}\n", ident, collection));
                for node in nodes {
                    node.as_string_impl(result, &format!("{}  ", ident));
                }
            },
        }
    }
}

impl TreeNode {
    pub fn from_pest_pairs(mut pairs: Pairs<Rule>) -> Result<TreeNode, TreeBuildError> {
        let children = parse_expression(pairs.next().expect("Expression is empty"))?;
        Ok(RootTreeNode { children }.into())
    }
}

// Returns children nodes generated from expression
fn parse_expression(pair: Pair<Rule>) -> Result<Vec<TreeNode>, TreeBuildError> {
    let pairs = pair.into_inner();
    parse_tail_expression(pairs)
}

// Returns children nodes generated from expression
fn parse_tail_expression(mut pairs: Pairs<Rule>) -> Result<Vec<TreeNode>, TreeBuildError> {
    let pair = if let Some(pair) = pairs.next() {
        pair
    } else {
        unreachable!("Empty expression is not possible");
    };

    let mut nodes = vec![];

    // Current node which we are processing
    let mut new_nodes = parse_term_any(pair)?;

    let mut current_node = new_nodes.pop().expect("term contain at least one node");
    // push all processed nodes in term
    nodes.extend(new_nodes);

    loop {
        match pairs.next().map(|p| p.as_rule()) {
            None => {
                nodes.push(current_node);
                break;
            },
            Some(Rule::sibling_op) => {
                let mut siblings =  parse_term_any(pairs.next().expect("Expression operator is missing"))?;
                let new_current_node = siblings.pop().expect("At least on sibling expected");
                nodes.push(std::mem::replace(&mut current_node, new_current_node));
                nodes.extend(siblings);
            }
            Some(Rule::child_op) => {
                let current_node_children = current_node.children_mut()?;
                let children = parse_tail_expression(pairs)?;
                current_node_children.extend(children);
                nodes.push(current_node);
                break;
            }
            r => unreachable!("Invalid expression operator {:?}", r),
        }
    }


    Ok(nodes)
}

fn parse_term_any(pair: Pair<Rule>) -> Result<Vec<TreeNode>, TreeBuildError> {
    let tree_node = match pair.as_rule() {
        Rule::term => parse_term(pair)?,
        Rule::term_list => parse_term_list(pair)?,
        e => unreachable!("expression produce only term or term_list, got {:?}", e),
    };
    Ok(tree_node)
}

fn parse_term(pair: Pair<Rule>) -> Result<Vec<TreeNode>, TreeBuildError> {
    let pairs = pair.into_inner().next().expect("Term have exact one pair");
    parse_term_content(pairs)
}

fn parse_term_content(pair: Pair<Rule>) -> Result<Vec<TreeNode>, TreeBuildError> {
    match pair.as_rule() {
        Rule::node => {
            Ok(vec![parse_node(pair)?])
        }
        Rule::text_node => Ok(parse_text_node(pair)?),
        Rule::node_binding => Ok(vec![parse_node_binding(pair)?]),
        Rule::expr => parse_expression(pair),
        e => unreachable!("Invalid term inner rule {:?}", e),
    }
}

fn parse_term_list(pair: Pair<Rule>) -> Result<Vec<TreeNode>, TreeBuildError> {
    let mut pairs = pair.into_inner(); // inner term_list components
    let term = pairs.next().expect("Term list should have term inside");
    let multiplier = pairs.next().expect("Term list should have multiplier");
    assert_eq!(multiplier.as_rule(), Rule::multiplier);

    let term_nodes = parse_term_content(term)?;
    let multiplier = multiplier.into_inner().next().expect("Multiplayer can't be empty");
    match multiplier.as_rule() {
        Rule::number => {
            let multiplier = parse_number(multiplier)?;
            let mut all_nodes = vec![];
            for _ in 0..multiplier {
                all_nodes.extend(term_nodes.clone());
            }
            Ok(all_nodes)
        }
        Rule::binding => {
            let binding = parse_binding(multiplier)?;
            Ok(vec![
                CollectionTreeNode {
                    nodes: term_nodes,
                    collection: binding,
                }.into()
            ])
        }
        r => unreachable!("Invalid multiplier rule: {:?}", r),
    }
}

fn parse_text_node(pair: Pair<Rule>) -> Result<Vec<TreeNode>, TreeBuildError> {
    let mut nodes = vec![];
    let mut first = true;
    // TODO: Optimize, merge text/number nodes into one string
    for pair in pair.into_inner() {
        if !first {
            // Push implicit space between text
            nodes.push(InnerContentTreeNode {
                value: GenericValue::Text(" ".to_owned()),
            }.into());
        }

        let node = InnerContentTreeNode {
            value: parse_generic_value(pair)?
        };
        nodes.push(node.into());

        first = false;
    }
    Ok(nodes)
}

fn parse_node_binding(pair: Pair<Rule>) -> Result<TreeNode, TreeBuildError> {
    let binding = parse_binding(pair)?;
    Ok(SubtreeTreeNode {
        property: binding,
    }.into())
}

fn parse_node(pair: Pair<Rule>) -> Result<TreeNode, TreeBuildError> {
    let mut pairs = pair.into_inner();
    let node_name = pairs.next().expect("Node always have name").as_str();
    let mut node = NormalTreeNode {
        name: node_name.to_owned(),
        attributes: vec![],
        children: vec![],
    };

    while let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::id_prop => {
                node.append_attribute("id", GenericValue::Text(pair.as_str().to_owned()));
            },
            Rule::class_prop => {
                node.append_attribute("class", GenericValue::Text(pair.as_str().to_owned()));
            },
            Rule::attrs_prop => {
                // For each Rule::attr
                for attr_pair in pair.into_inner() {
                    let mut attr_parts = attr_pair.into_inner();
                    let name = attr_parts.next().expect("Attr name always exist").as_str();
                    let value = parse_generic_value(
                        attr_parts.next().expect("Attr always have value")
                            .into_inner().next().expect("Attr value always have inner item")
                    )?;
                    node.append_attribute(name, value);
                }
            }
            _ => unreachable!("Invalid node property"),
        }
    }

    Ok(node.into())
}

fn parse_generic_value(pair: Pair<Rule>) -> Result<GenericValue, TreeBuildError> {
    let value = match pair.as_rule() {
        Rule::ident => GenericValue::Text(pair.as_str().to_owned()),
        Rule::number => GenericValue::Number(parse_number(pair)?),
        Rule::string => GenericValue::Text(parse_string(pair)?),
        Rule::binding => GenericValue::Binding(parse_binding(pair)?),
        _ => unreachable!("Invalid generic values")
    };

    Ok(value)
}

fn parse_number(pair: Pair<Rule>) -> Result<isize, TreeBuildError> {
    pair.as_str().parse().map_err(|_| TreeBuildError::InvalidNumLiteral)
}

fn parse_string(pair: Pair<Rule>) -> Result<String, TreeBuildError> {
    Ok(pair.into_inner().next().expect("String always have inner part").as_str().to_owned())
}

fn parse_binding(pair: Pair<Rule>) -> Result<PropertyBinding, TreeBuildError> {
    let inner = pair.into_inner();
    let mut parts = vec![];
    for pair in inner {
        assert_eq!(pair.as_rule(), Rule::binding_part);
        parts.push(pair.as_str().to_owned());
    }

    let binding = if parts.len() == 1 {
        PropertyBinding::RootIdentifier(parts.pop().unwrap())
    } else {
        PropertyBinding::NestedIdentifier(parts)
    };

    Ok(binding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use expect_test::{expect, Expect};
    use crate::parser::SyntaxParser;
    use pest::Parser;

    fn assert_parsed(input: &str, expect: Expect) {
        let pairs = SyntaxParser::parse(Rule::tree, input).unwrap();
        let tree = TreeNode::from_pest_pairs(pairs).unwrap();
        expect.assert_eq(&tree.to_string());
    }

    #[test]
    fn smoke_test() {
        assert_parsed(r#"a#id.class1.class2
            >b
            +c[one=two two=22 two='test']
                >d[test=@binding test2=@nested%binding]
        "#,
        expect![[r#"
             - a[id="id" class={"class1", "class2"}]
               - b
               - c[one="two" two={22, "test"}]
                 - d[test=@binding test2=@nested%binding]
        "#]]);

        assert_parsed("a+(b>c)+d",
    expect![[r#"
         - a
         - b
           - c
         - d
    "#]]
        );

        assert_parsed("(b>c + d)>a",
    expect![[r#"
         - b
           - c
           - d
           - a
    "#]]
        );

        assert_parsed("div>{my 'text' 42 @binding @a%b%c}",
            expect![[r#"
                 - div
                   - [CONTENT] "my"
                   - [CONTENT] " "
                   - [CONTENT] "text"
                   - [CONTENT] " "
                   - [CONTENT] 42
                   - [CONTENT] " "
                   - [CONTENT] @binding
                   - [CONTENT] " "
                   - [CONTENT] @a%b%c
            "#]]
        );

        assert_parsed("div#test>$sub%tree+a+{hello}",
            expect![[r#"
                 - div[id="test"]
                   - [SUBTREE] @sub%tree
                   - a
                   - [CONTENT] "hello"
            "#]]
        );

        assert_parsed("li.item * 5",
            expect![[r#"
                 - li[class="item"]
                 - li[class="item"]
                 - li[class="item"]
                 - li[class="item"]
                 - li[class="item"]
            "#]]
        );

        assert_parsed("(li[id=@c%id]>b>{Item @c%name}) * @c + div",
        expect![[r#"
             - [COLLECTION] @c
               - li[id=@c%id]
                 - b
                   - [CONTENT] "Item"
                   - [CONTENT] " "
                   - [CONTENT] @c%name
             - div
        "#]]
    );
    }
}
