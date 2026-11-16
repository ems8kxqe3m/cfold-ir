//! Constant folding: replace instructions whose operands are both known
//! constants by the folded constant, then clean up algebraic identities.
//!
//! The pass runs to a fixpoint. An identity such as `x + 0` can turn an operand
//! into a constant, which may make the *next* instruction foldable, and an
//! ordering-sensitive caller should not have to know that. Each sweep that
//! removes at least one instruction is followed by another; a sweep that removes
//! nothing terminates the pass.

use std::collections::HashMap;

use crate::ir::{Func, Op, ValueId};

/// Fold `f` in place. Returns the number of instructions removed.
pub fn fold(f: &mut Func) -> usize {
    let mut removed = 0;

    loop {
        // Value id -> replacement produced earlier in this same sweep.
        let mut subst: HashMap<ValueId, ValueId> = HashMap::new();
        let mut round = 0;

        for idx in 0..f.insts.len() {
            let op = f.insts[idx].op;
            let lhs = resolve(&subst, f.insts[idx].lhs);
            let rhs = resolve(&subst, f.insts[idx].rhs);

            // Canonical operand order: constants go right, which is the shape
            // every identity rule below is written against.
            let (lhs, rhs) = if op.is_commutative() && f.const_of(lhs).is_some() && f.const_of(rhs).is_none()
            {
                (rhs, lhs)
            } else {
                (lhs, rhs)
            };
            f.insts[idx].lhs = lhs;
            f.insts[idx].rhs = rhs;

            let replacement = match (f.const_of(lhs), f.const_of(rhs)) {
                (Some(a), Some(b)) => op.eval(a, b).map(|value| f.const_id(value)),
                _ => simplify(f, op, lhs, rhs),
            };

            if let Some(replacement) = replacement {
                subst.insert(f.insts[idx].id, replacement);
                round += 1;
            }
        }

        if round == 0 {
            return removed;
        }
        removed += round;

        f.insts.retain(|inst| !subst.contains_key(&inst.id));
        for idx in 0..f.insts.len() {
            f.insts[idx].lhs = resolve(&subst, f.insts[idx].lhs);
            f.insts[idx].rhs = resolve(&subst, f.insts[idx].rhs);
        }
        let ret = resolve(&subst, f.ret);
        f.ret = ret;
    }
}

/// Follow the replacement chain for `value`.
fn resolve(subst: &HashMap<ValueId, ValueId>, mut value: ValueId) -> ValueId {
    while let Some(&next) = subst.get(&value) {
        value = next;
    }
    value
}

/// Algebraic identities that shrink an instruction without needing both
/// operands to be constants. `rhs` is constant whenever a rule checks for one,
/// thanks to the canonical operand order in [`fold`].
fn simplify(f: &mut Func, op: Op, lhs: ValueId, rhs: ValueId) -> Option<ValueId> {
    let rhs_const = f.const_of(rhs);

    match op {
        Op::Add | Op::Or | Op::Xor if rhs_const == Some(0) => Some(lhs),
        Op::Sub | Op::Shl | Op::Shr if rhs_const == Some(0) => Some(lhs),
        Op::Mul if rhs_const == Some(1) => Some(lhs),
        Op::Mul | Op::And if rhs_const == Some(0) => Some(f.const_id(0)),
        Op::And | Op::Or if lhs == rhs => Some(lhs),
        Op::Xor | Op::Sub if lhs == rhs => Some(f.const_id(0)),
        _ => None,
    }
}
