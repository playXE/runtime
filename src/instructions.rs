#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Instruction {
    LdInt(i64),
    LdFloat(u64 /* f64::to_bits() && f64::from_bits() */),
    LdGlobal(u32),
    LdLocal(u32),
    LdEnv(u32),
    LdStatic(u32),
    LdField,
    StLocal(u32),
    StEnv(u32),
    StStatic(u32),
    StField,

    TailCall(u32),
    Call(u32),

    ThreadYield,

    Jmp(u32),
    JmpZ(u32),
    JmpNz(u32),

    Add,
    Sub,
    Div,
    Mul,
    Mod,
    Shr,
    Shl,
    Pop(u32),
    Dup,
}

impl Instruction {
    pub fn can_observe_side_effects(&self) -> bool {
        use Instruction::*;
        match self {
            StEnv(_) | StField | StLocal(_) | StStatic(_) | LdEnv(_) | LdField | LdGlobal(_)
            | LdLocal(_) | LdStatic(_) => true,
            _ => false,
        }
    }
}
