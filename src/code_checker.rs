use std::ops::Add;

use crate::function::FlattenedInstruction;
use crate::variable::Variable;

use anyhow::{anyhow, Context, Result};
use either::Either;

/* structure permettant de suivre au cours des instructions le positionnement de la tête de lecture */
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Outputpointer {
    Predictable(isize),
    Unpredictable,
}

impl Outputpointer {
    fn get_value(&self) -> Result<isize> {
        match self {
            Outputpointer::Predictable(val) => Ok(*val),
            _ => Err(anyhow!("try to unwrap an unpredictable pointer")),
        }
    }
}

impl From<isize> for Outputpointer {
    fn from(t: isize) -> Self {
        Self::Predictable(t)
    }
}

impl Default for Outputpointer {
    fn default() -> Self {
        Outputpointer::Predictable(0)
    }
}

impl Add for Outputpointer {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Outputpointer::Predictable(p1), Outputpointer::Predictable(p2)) => {
                Outputpointer::Predictable(p1 + p2)
            }
            _ => Outputpointer::Unpredictable,
        }
    }
}

/* fonction permettant de faire évoluer au cours d'une suite d'instruction la position de la tête de lecture */
fn predict_position(
    ist_list: &Vec<FlattenedInstruction>,
    start_pos: Outputpointer,
) -> Outputpointer {
    ist_list
        .iter()
        .fold(Outputpointer::default(), |acc, ist| match ist {
            FlattenedInstruction::Left(val) => acc + ((-(*val as isize)).into()),
            FlattenedInstruction::Right(val) => acc + (*val as isize).into(),
            FlattenedInstruction::Loop(inner) => {
                if predict_position(&inner, acc) == acc {
                    acc
                } else {
                    Outputpointer::Unpredictable
                }
            }
            _ => acc,
        })
}

/* fonction permettant de faire évoluer au cours d'une instruction la position de la tête de lecture */
fn add_position_single(ist: &FlattenedInstruction, start_pos: Outputpointer) -> Outputpointer {
    match ist {
        FlattenedInstruction::Left(val) => start_pos + ((-(*val as isize)).into()),
        FlattenedInstruction::Right(val) => start_pos + (*val as isize).into(),
        FlattenedInstruction::Loop(inner) => {
            if predict_position(&inner, start_pos) == start_pos {
                start_pos
            } else {
                Outputpointer::Unpredictable
            }
        }
        _ => start_pos,
    }
}

/* fonction permettant de faire deux chose:
    1) de vérifier que l'on appelle Goto uniquement lorsque l'on connait la position de la tête de lecture, par exemple, [>] fait perdre la connaissance de cette position
    2) dans le cas où le goto est valide, elle permet de remplacer ce goto en left ou right en fonction de la position de la variable
*/
pub fn transform_goto(
    ist_list: Vec<FlattenedInstruction>,
    start_pos: Outputpointer,
) -> (Result<Vec<FlattenedInstruction>>, Outputpointer) {
    let mut position = start_pos;
    (
        ist_list
            .into_iter()
            .map(|ist| {
                position = add_position_single(&ist, position);
                match ist {
                    FlattenedInstruction::Goto(to_pos) => {
                        let actual_pos = position.get_value()?;
                        if to_pos as isize == actual_pos {
                            Ok(FlattenedInstruction::Noop)
                        } else if (to_pos as isize) < actual_pos {
                            Ok(FlattenedInstruction::Left(
                                (actual_pos - (to_pos as isize)).try_into().unwrap(),
                            ))
                        } else {
                            Ok(FlattenedInstruction::Right(
                                ((to_pos as isize) - actual_pos).try_into().unwrap(),
                            ))
                        }
                    }
                    FlattenedInstruction::Loop(inner) => {
                        let (res, position) = transform_goto(inner, position);
                        Ok(FlattenedInstruction::Loop(res?))
                    }
                    e => {
                        position = add_position_single(&e, position);
                        Ok(e)
                    }
                }
            })
            .collect::<Result<Vec<_>>>(),
        position,
    )
}
