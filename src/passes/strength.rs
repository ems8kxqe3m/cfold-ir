//! Strength reduction: rewrite multiply into a shift when the multiplier is a
//! power of two, and `x + x` into `shl x, 1`.
//!
//! The pass reads the constant operand from the right-hand side, so it is meant
//! to run after [`crate::passes::fold::fold`], which canonicalises commutative
//! operands. It touches no value other than the rewritten instruction, which
//! keeps it cheap enough to run on every function in a module.

use crate::ir::{Func, Op};

/// Rewrite the instructions of `f` in place. Returns the number of instructions
/// rewritten.
pub fn reduce(f: &mut Func) -> usize {
    // Plan first: interning the shift amount needs `&mut Func` while walking the
    // instruction list.
    let mut plans: Vec<(usize, u32)> = Vec::new();

    for (idx, inst) in f.insts.iter().enumerate() {
        match (inst.op, f.const_of(inst.rhs)) {
            (Op::Mul, Some(factor)) if factor > 1 && (factor as u64).is_power_of_two() => {
                plans.push((idx, (factor as u64).trailing_zeros()));
            }
            (Op::Add, _) if inst.lhs == inst.rhs => plans.push((idx, 1)),
            _ => {}
        }
    }

    for &(idx, shift) in &plans {
        let amount = f.const_id(i64::from(shift));
        let inst = &mut f.insts[idx];
        inst.op = Op::Shl;
        inst.rhs = amount;
    }

    plans.len()
}
