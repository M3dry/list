use crate::{
    parser::{Parser, ParserError},
    tokenizer::{Literals, Types},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PatternTree {
    value: PatternToken,
    nodes: Vec<Self>,
}

impl TryFrom<&mut Parser> for PatternTree {
    type Error = ParserError;

    fn try_from(value: &mut Parser) -> Result<Self, Self::Error> {
        Ok(PatternTree::new(PatternToken::Literal(Literals::String(
            "Hello".to_string(),
        ))))
    }
}

impl PatternTree {
    pub(crate) fn new(value: PatternToken) -> Self {
        Self {
            value,
            nodes: vec![],
        }
    }

    pub(crate) fn nodes(mut self, nodes: Vec<Self>) -> Self {
        self.nodes = nodes;
        self
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PatternToken {
    Literal(Literals),
    Type(Types),
    Identifier(String),
}
