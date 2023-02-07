use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use pest::iterators::Pair;

use crate::instruction::Rule;

/* structure permettant de représenter les arguments des instructions pouvant être des littéraux dans le cas d'une fonction */
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Variable {
    Named(String),
    Constant(usize),
}
impl Variable {
    pub fn parse(entry: Pair<Rule>) -> Result<Self> {
        match entry.as_rule() {
            Rule::number => {
                let content = entry.as_span().as_str();
                Ok(Self::Constant(content.parse()?))
            }
            Rule::name => {
                let content = entry.as_span().as_str();
                Ok(Self::Named(content.to_string()))
            }
            _ => Err(anyhow!("pass an non-variable pair into variable parser")),
        }
    }

    #[inline]
    pub fn parse_vec(entry: Pair<Rule>) -> Vec<Self> {
        entry
            .into_inner()
            .map(|arg| Self::parse(arg).expect("failed to parse one argument"))
            .collect::<Vec<_>>()
    }

    /* fonction permettant de retourner une copie de la valeur numérique de la variable dans le cas d'une variable avec une valeur numérique */
    #[inline]
    pub fn unwrap_value(&self) -> usize {
        match self {
            Self::Constant(value) => *value,
            _ => panic!("failed to unwrap name"),
        }
    }

    pub fn copy(&self) -> Self {
        match self {
            Self::Named(name) => Self::Named(name.to_owned()),
            Self::Constant(u) => Self::Constant(*u),
        }
    }

    /* fonction permettant de substituer une variable grâce à une table de correspondance */
    pub fn try_substitute(&self, mapping: &HashMap<String, Variable>) -> Result<Self> {
        match self {
            Self::Constant(u) => Ok(Self::Constant(*u)),
            Self::Named(name) => {
                let n = mapping
                    .get(name)
                    .context("try to use an undeclared variable")?;

                Ok(n.clone())
            }
        }
    }
}
