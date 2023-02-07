use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use pest::Parser;
extern crate pest;
use pest::iterators::Pair;

use crate::function::FnSignature;
use crate::optimizer::InstructionKind;
use crate::variable::Variable;
use crate::STD_FUNCTION;

#[derive(Parser)]
#[grammar = "bf_il.pest"]
struct BFILParser;

#[derive(Debug)]
pub enum Instruction {
    Noop,
    Print,
    Add(Variable),
    Sub(Variable),
    SetConst(Variable),
    Goto(Variable),
    Left(Variable),
    Right(Variable),
    Reset,
    FnCall(String, Vec<Variable>), //first: name function second: arg
    Fn(String, FnSignature, Vec<Instruction>), //nom, contenue
    Loop(Vec<Instruction>),
}

impl Instruction {
    /* fonction permettant de parser les instctions */
    fn from(source: Pair<Rule>, function_namespace: &mut HashSet<String>) -> Result<Self> {
        match source.as_rule() {
            Rule::Loop => {
                let ist_inner = source.into_inner();
                Ok(Self::Loop(
                    ist_inner
                        .map(|ist| Self::from(ist, function_namespace))
                        .collect::<Result<Vec<_>>>()
                        .context("failed to parse function instruction")?,
                ))
            }

            Rule::Function => {
                let mut ist_inner = source.into_inner();
                let fucname = ist_inner
                    .next()
                    .context(format!("failed to parse function name on {:?}", ist_inner))?
                    .as_str();

                if function_namespace.contains(fucname) || STD_FUNCTION.contains(&fucname) {
                    Err(anyhow!(
                        "try to redeclare function with is already declared"
                    ))
                } else {
                    let funcarg = ist_inner
                        .next()
                        .context(format!(
                            "failed to parse function argument on {:?}",
                            ist_inner
                        ))?
                        .into_inner()
                        .map(|arg| match arg.as_rule() {
                            Rule::name => Ok(arg.as_str().to_owned()),
                            _ => Err(anyhow!("pass an non-variable pair into variable parser")),
                        })
                        .collect::<Result<Vec<_>>>()?;

                    function_namespace.insert(fucname.to_string());

                    let signature = FnSignature::from(funcarg);

                    let funcist = ist_inner
                        .map(|ist| Self::from(ist, function_namespace))
                        .collect::<Result<Vec<_>>>()
                        .context("failed to parse function instruction")?;

                    Ok(Self::Fn(fucname.to_string(), signature, funcist))
                }
            }
            Rule::Loop => {
                let mut ist_inner = source.into_inner();
                let loopist = ist_inner
                    .map(|ist| Self::from(ist, function_namespace))
                    .collect::<Result<Vec<_>>>()
                    .context("failed to parse function instruction")?;

                Ok(Self::Loop(loopist))
            }

            Rule::Instruction => {
                let mut ist_inner = source.into_inner();
                let ist_name = ist_inner
                    .next()
                    .context(format!("failed to parse function name on {:?}", ist_inner))?
                    .as_str()
                    .to_lowercase();

                match ist_name.as_ref() {
                    "goto" => {
                        let mut var = Variable::parse_vec(
                            ist_inner.next().context("failed to parse argument")?,
                        );
                        if var.len() == 1 {
                            let ist_var = var.pop().unwrap();
                            Ok(Instruction::Goto(ist_var))
                        } else {
                            Err(anyhow!(format!(
                                "invalid argument in goto call, expected one, get {}",
                                var.len()
                            )))
                        }
                    }
                    "add" => {
                        let mut var = Variable::parse_vec(
                            ist_inner.next().context("failed to parse argument")?,
                        );
                        if var.len() == 1 {
                            let ist_var = var.pop().unwrap();
                            Ok(Instruction::Add(ist_var))
                        } else {
                            Err(anyhow!(format!(
                                "invalid argument in add call, expected one, get {}",
                                var.len()
                            )))
                        }
                    }
                    "sub" => {
                        let mut var = Variable::parse_vec(
                            ist_inner.next().context("failed to parse argument")?,
                        );
                        if var.len() == 1 {
                            let ist_var = var.pop().unwrap();
                            Ok(Instruction::Sub(ist_var))
                        } else {
                            Err(anyhow!(format!(
                                "invalid argument in Sub call, expected one, get {}",
                                var.len()
                            )))
                        }
                    }
                    "left" => {
                        let mut var = Variable::parse_vec(
                            ist_inner.next().context("failed to parse argument")?,
                        );
                        if var.len() == 1 {
                            let ist_var = var.pop().unwrap();
                            Ok(Instruction::Left(ist_var))
                        } else {
                            Err(anyhow!(format!(
                                "invalid argument in Left call, expected one, get {}",
                                var.len()
                            )))
                        }
                    }
                    "right" => {
                        let mut var = Variable::parse_vec(
                            ist_inner.next().context("failed to parse argument")?,
                        );
                        if var.len() == 1 {
                            let ist_var = var.pop().unwrap();
                            Ok(Instruction::Right(ist_var))
                        } else {
                            Err(anyhow!(format!(
                                "invalid argument in Right call, expected one, get {}",
                                var.len()
                            )))
                        }
                    }
                    "setconst" => {
                        let mut var = Variable::parse_vec(
                            ist_inner.next().context("failed to parse argument")?,
                        );
                        if var.len() == 1 {
                            let ist_var = var.pop().unwrap();
                            Ok(Instruction::SetConst(ist_var))
                        } else {
                            Err(anyhow!(format!(
                                "invalid argument in SetConst call, expected one, get {}",
                                var.len()
                            )))
                        }
                    }
                    "print" => {
                        if ist_inner.peek().is_none() {
                            Ok(Instruction::Print)
                        } else {
                            Err(anyhow!(format!(
                                "invalid argument in SetConst call, expected 0",
                            )))
                        }
                    }
                    "erbset" => {
                        if ist_inner.peek().is_none() {
                            Ok(Instruction::Reset)
                        } else {
                            Err(anyhow!(format!(
                                "invalid argument in Reset call, expected 0",
                            )))
                        }
                    }
                    name => Ok(Instruction::FnCall(
                        name.to_string(),
                        Variable::parse_vec(ist_inner.next().context("failed to parse argument")?),
                    )),
                }
            }

            _ => Ok(Self::Noop),
        }
    }

    pub fn parse(source: String) -> (Result<Vec<Instruction>>, HashSet<String>) {
        let mut function_namespace = HashSet::<String>::new();

        (
            BFILParser::parse(Rule::File, &source)
                .unwrap()
                .next()
                .unwrap()
                .into_inner()
                .map(|ist| Instruction::from(ist, &mut function_namespace))
                .collect::<Result<Vec<_>>>(),
            function_namespace,
        )
    }

    #[inline]
    pub const fn get_type(&self) -> InstructionKind {
        match self {
            Self::Add(_) | Self::Sub(_) | Self::Left(_) | Self::Right(_) => InstructionKind::Linear,
            Self::Noop | Self::SetConst(_) | Self::Goto(_) => InstructionKind::Replaceable,
            _ => InstructionKind::Any,
        }
    }
}
