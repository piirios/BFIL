use crate::function::FlattenedInstruction;
use std::collections::VecDeque;

pub enum InstructionKind {
    Linear,
    Replaceable,
    Any,
}
/* fonction permettant d'optimiser en modifiant dans certain cas deux instruction succéssive   */
pub fn optimize_consecutive(ist_list: Vec<FlattenedInstruction>) -> Vec<FlattenedInstruction> {
    let mut res = VecDeque::new();

    for ist in ist_list {
        if res.is_empty() {
            res.push_back(ist);
        } else {
            match (ist, res.pop_back().unwrap()) {
                (FlattenedInstruction::Add(v1), FlattenedInstruction::Add(v2)) => {
                    res.push_back(FlattenedInstruction::Add(v1 + v2));
                }
                (FlattenedInstruction::Sub(v1), FlattenedInstruction::Sub(v2)) => {
                    res.push_back(FlattenedInstruction::Sub(v1 + v2));
                }
                (FlattenedInstruction::Left(v1), FlattenedInstruction::Left(v2)) => {
                    res.push_back(FlattenedInstruction::Left(v1 + v2));
                }
                (FlattenedInstruction::Right(v1), FlattenedInstruction::Right(v2)) => {
                    res.push_back(FlattenedInstruction::Right(v1 + v2));
                }
                (FlattenedInstruction::Noop, FlattenedInstruction::Noop) => {
                    res.push_back(FlattenedInstruction::Noop);
                }
                (FlattenedInstruction::Goto(val), FlattenedInstruction::Goto(_)) => {
                    res.push_back(FlattenedInstruction::Goto(val));
                }
                (FlattenedInstruction::SetConst(val), FlattenedInstruction::SetConst(_)) => {
                    res.push_back(FlattenedInstruction::SetConst(val));
                }
                (FlattenedInstruction::Loop(inner), e) => {
                    res.push_back(e);
                    res.push_back(FlattenedInstruction::Loop(optimize_consecutive(inner)))
                }
                (e, v) => {
                    res.push_back(v);
                    res.push_back(e);
                }
            }
        }
    }

    res.into()
}
