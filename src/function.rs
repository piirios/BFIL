use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;

use anyhow::{anyhow, Context, Result};

use crate::instruction::Instruction;
use crate::variable::Variable;

/* après que l'on remplace les fonctions, on n'a plus besoins de certain champs, donc on utilise une nouvelle structure */
#[derive(Debug)]
pub enum FlattenedInstruction {
    Noop,
    Print,
    Add(usize),
    Sub(usize),
    SetConst(usize),
    Goto(usize),
    Left(usize),
    Right(usize),
    Loop(Vec<FlattenedInstruction>),
}

impl TryFrom<Instruction> for FlattenedInstruction {
    type Error = anyhow::Error;

    fn try_from(value: Instruction) -> Result<Self, Self::Error> {
        match value {
            Instruction::Noop => Ok(FlattenedInstruction::Noop),
            Instruction::Print => Ok(FlattenedInstruction::Print),
            Instruction::Reset => Ok(Self::Loop(vec![Self::Sub(1)])),
            Instruction::Add(v) => Ok(Self::Add(v.unwrap_value())),
            Instruction::Sub(v) => Ok(Self::Sub(v.unwrap_value())),
            Instruction::SetConst(v) => Ok(Self::SetConst(v.unwrap_value())),
            Instruction::Goto(v) => Ok(Self::Goto(v.unwrap_value())),
            Instruction::Left(v) => Ok(Self::Left(v.unwrap_value())),
            Instruction::Right(v) => Ok(Self::Right(v.unwrap_value())),
            Instruction::Loop(inner) => Ok(Self::Loop(
                inner
                    .into_iter()
                    .map(|ist| Self::try_from(ist))
                    .collect::<Result<Vec<_>>>()?,
            )),
            _ => unreachable!(),
        }
    }
}

/* structure permettant de garder en mémoire les signatures des fonctions */
#[derive(Debug)]
pub struct FnSignature {
    arg_state: Vec<String>,
}

impl From<Vec<String>> for FnSignature {
    fn from(src: Vec<String>) -> Self {
        Self { arg_state: src }
    }
}

impl FnSignature {
    /* fonction permettant de produire un mapping à partir des valeurs de chaque argument */
    fn produce_args_mapping(&self, args: Vec<Variable>) -> HashMap<String, Variable> {
        self.arg_state
            .iter()
            .zip(args.into_iter())
            .map(|(name, value)| (name.to_owned(), value))
            .collect::<HashMap<_, _>>()
    }
}

/* fontion permettant de faire deux chose:
    1) vérifier que l'on appele pas des fonction non déclarés
    2) sépare les fonctions du reste des instructions
*/
pub fn produce_mapping(
    ist_list: Vec<Instruction>,
    namespace: HashSet<String>,
) -> Result<(
    Vec<Instruction>,
    HashMap<String, (FnSignature, Vec<Instruction>)>,
)> {
    let mut error_calling = String::default();

    let (fn_list, ist_without_list): (Vec<_>, Vec<_>) = ist_list
        .into_iter()
        .partition(|ist| matches!(ist, Instruction::Fn(_, _, _)));

    let mapping = fn_list
        .into_iter()
        .map(|function| match function {
            Instruction::Fn(name, sign, content) => (name, (sign, content)),
            _ => unreachable!(),
        })
        .collect::<HashMap<_, _>>();

    ist_without_list.iter().for_each(|ist| {
        match ist {
            Instruction::FnCall(name, _) => {
                if !namespace.contains(name) {
                    error_calling = name.to_owned();
                }
            }
            _ => (),
        };
    });

    if error_calling != String::default() {
        Err(anyhow!(format!(
            "try to call a non-declared function: {}",
            error_calling
        )))
    } else {
        Ok((ist_without_list, mapping))
    }
}

/* fonction permettant de subsituter toute les varibles littérale en leurs valeurs dans une appels de fonction
*/
pub fn substitute(
    ist_list: &Vec<Instruction>,
    substitution: &HashMap<String, Variable>,
) -> Result<Vec<Instruction>> {
    ist_list
        .iter()
        .map(|ist| match ist {
            Instruction::Noop => Ok(Instruction::Noop),
            Instruction::Print => Ok(Instruction::Print),
            Instruction::Reset => Ok(Instruction::Reset),
            Instruction::Add(var) => Ok(Instruction::Add(var.try_substitute(substitution)?)),
            Instruction::Sub(var) => Ok(Instruction::Sub(var.try_substitute(substitution)?)),
            Instruction::Left(var) => Ok(Instruction::Left(var.try_substitute(substitution)?)),
            Instruction::Right(var) => Ok(Instruction::Right(var.try_substitute(substitution)?)),
            Instruction::SetConst(var) => {
                Ok(Instruction::SetConst(var.try_substitute(substitution)?))
            }
            Instruction::Goto(var) => Ok(Instruction::Goto(var.try_substitute(substitution)?)),
            Instruction::Loop(inner) => Ok(Instruction::Loop(substitute(&inner, substitution)?)),
            Instruction::FnCall(name, args) => Ok(Instruction::FnCall(
                name.to_owned(),
                args.into_iter()
                    .map(|arg| arg.try_substitute(substitution))
                    .collect::<Result<Vec<_>>>()?,
            )),
            Instruction::Fn(_, _, _) => unreachable!(),
        })
        .collect::<Result<Vec<_>>>()
}

/* fonction permettant de subsituter tout les appels de fonction en leurs contenue, ici on interdit la récursivité car les ce que l'on appele ici fonction
    n'est qu'ne soit des macros
*/
pub fn replace_function(
    ist_list: &Vec<Instruction>,
    mapping: &HashMap<String, (FnSignature, Vec<Instruction>)>,
    context: &mut HashSet<String>,
) -> Result<Vec<FlattenedInstruction>> {
    Ok(ist_list
        .into_iter()
        .map(|ist| {
            match ist {
                Instruction::FnCall(fn_name, args) => {
                    let (sign, fun_ist_list) = mapping.get(fn_name).unwrap(); //on ne peut pas fail l'unwrap car on a déjà

                    if context.contains(fn_name) {
                        Err(anyhow!(format!(
                            "try to recall an already called function {}",
                            fn_name
                        )))
                    } else {
                        let substitution = sign.produce_args_mapping(args.to_vec());

                        let res = substitute(fun_ist_list, &substitution)?;
                        context.insert(fn_name.to_owned());
                        replace_function(&res, mapping, context)
                    }
                }
                Instruction::Noop => Ok(vec![FlattenedInstruction::Noop]),
                Instruction::Print => Ok(vec![FlattenedInstruction::Print]),
                Instruction::Reset => Ok(vec![FlattenedInstruction::Loop(vec![
                    FlattenedInstruction::Sub(1),
                ])]),
                Instruction::Add(var) => Ok(vec![FlattenedInstruction::Add(var.unwrap_value())]),
                Instruction::Sub(var) => Ok(vec![FlattenedInstruction::Sub(var.unwrap_value())]),
                Instruction::Left(var) => Ok(vec![FlattenedInstruction::Left(var.unwrap_value())]),
                Instruction::Right(var) => {
                    Ok(vec![FlattenedInstruction::Right(var.unwrap_value())])
                }
                Instruction::SetConst(var) => {
                    Ok(vec![FlattenedInstruction::SetConst(var.unwrap_value())])
                }
                Instruction::Goto(var) => Ok(vec![FlattenedInstruction::Goto(var.unwrap_value())]),

                Instruction::Loop(inner) => Ok(vec![FlattenedInstruction::Loop(replace_function(
                    inner, mapping, context,
                )?)]),

                Instruction::Fn(_, _, _) => unreachable!(),
            }
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect())
}
