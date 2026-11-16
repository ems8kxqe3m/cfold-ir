//! A deliberately small, single-block SSA IR over `i64`.
//!
//! Values are numbered. Ids below `params.len()` are function parameters; every
//! other id must be defined by an instruction or by the constant table.
//! Instructions are stored in dominance order, so a value is always defined
//! before any use, which is what lets the folding pass run as a single ordered
//! sweep.

use std::collections::BTreeMap;

pub type ValueId = usize;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Lt,
    Eq,
}

impl Op {
    pub fn mnemonic(self) -> &'static str {
        match self {
            Op::Add => "add",
            Op::Sub => "sub",
            Op::Mul => "mul",
            Op::And => "and",
            Op::Or => "or",
            Op::Xor => "xor",
            Op::Shl => "shl",
            Op::Shr => "shr",
            Op::Lt => "lt",
            Op::Eq => "eq",
        }
    }

    /// Ops whose operand order carries no meaning. The folder sorts these so a
    /// constant always ends up on the right-hand side.
    pub fn is_commutative(self) -> bool {
        matches!(self, Op::Add | Op::Mul | Op::And | Op::Or | Op::Xor | Op::Eq)
    }

    /// Evaluate the op on two constants. Returns `None` when the result is not
    /// representable, so the caller keeps the instruction instead of guessing.
    pub fn eval(self, lhs: i64, rhs: i64) -> Option<i64> {
        match self {
            Op::Add => lhs.checked_add(rhs),
            Op::Sub => lhs.checked_sub(rhs),
            Op::Mul => lhs.checked_mul(rhs),
            Op::And => Some(lhs & rhs),
            Op::Or => Some(lhs | rhs),
            Op::Xor => Some(lhs ^ rhs),
            Op::Shl if (0..64).contains(&rhs) => Some(lhs << rhs),
            Op::Shr if (0..64).contains(&rhs) => Some(lhs >> rhs),
            Op::Lt => Some(i64::from(lhs < rhs)),
            Op::Eq => Some(i64::from(lhs == rhs)),
            Op::Shl | Op::Shr => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Inst {
    pub id: ValueId,
    pub op: Op,
    pub lhs: ValueId,
    pub rhs: ValueId,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub name: String,
    pub params: Vec<ValueId>,
    consts: BTreeMap<ValueId, i64>,
    pub insts: Vec<Inst>,
    pub ret: ValueId,
    next_id: ValueId,
}

impl Func {
    pub fn new(name: &str, arity: usize) -> Self {
        Self {
            name: name.to_string(),
            params: (0..arity).collect(),
            consts: BTreeMap::new(),
            insts: Vec::new(),
            ret: 0,
            next_id: arity,
        }
    }

    /// Intern `value` and return its id. Repeated constants share one id.
    pub fn const_id(&mut self, value: i64) -> ValueId {
        if let Some((&id, _)) = self.consts.iter().find(|(_, v)| **v == value) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.consts.insert(id, value);
        id
    }

    pub fn const_of(&self, value: ValueId) -> Option<i64> {
        self.consts.get(&value).copied()
    }

    pub fn inst(&self, id: ValueId) -> Option<&Inst> {
        self.insts.iter().find(|inst| inst.id == id)
    }

    /// Append `op lhs, rhs` and return the id of the new value.
    pub fn bin(&mut self, op: Op, lhs: ValueId, rhs: ValueId) -> ValueId {
        let id = self.next_id;
        self.next_id += 1;
        self.insts.push(Inst { id, op, lhs, rhs });
        id
    }

    pub fn set_ret(&mut self, value: ValueId) {
        self.ret = value;
    }

    fn operand(&self, value: ValueId) -> String {
        if let Some(constant) = self.const_of(value) {
            format!("c{constant}")
        } else if self.params.contains(&value) {
            format!("%p{value}")
        } else {
            format!("%v{value}")
        }
    }
}

impl std::fmt::Display for Func {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<String> = self
            .params
            .iter()
            .map(|param| format!("%p{param}: i64"))
            .collect();
        writeln!(f, "fn {}({}) -> i64 {{", self.name, params.join(", "))?;
        for inst in &self.insts {
            writeln!(
                f,
                "  %v{} = {} {}, {}",
                inst.id,
                inst.op.mnemonic(),
                self.operand(inst.lhs),
                self.operand(inst.rhs)
            )?;
        }
        writeln!(f, "  ret {}", self.operand(self.ret))?;
        write!(f, "}}")
    }
}
