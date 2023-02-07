use crate::function::FlattenedInstruction;

/* fonction permettant de transformer chaque instruction en chaine de caractère finale en Brainfuck
    ici on suppose les goto déjà subsituter en Left ou Right
*/
pub fn produce_string(ist: FlattenedInstruction) -> String {
    match ist {
        FlattenedInstruction::Noop => String::default(),
        FlattenedInstruction::Print => String::from(","),
        FlattenedInstruction::Add(val) => "+".repeat(val),
        FlattenedInstruction::Sub(val) => "-".repeat(val),
        FlattenedInstruction::Left(val) => "<".repeat(val),
        FlattenedInstruction::Right(val) => ">".repeat(val),
        FlattenedInstruction::SetConst(val) => {
            let mut s = String::from("[-]");
            s.push_str(&"+".repeat(val));
            s
        }
        FlattenedInstruction::Loop(inner) => {
            let mut s = String::from("[");
            s.push_str(
                &inner
                    .into_iter()
                    .map(|ist| produce_string(ist))
                    .collect::<String>(),
            );
            s.push_str("]");
            s
        }
        _ => unreachable!(),
    }
}
