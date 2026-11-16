use cfold_ir::ir::{Func, Op};
use cfold_ir::passes::fold::fold;

fn show(f: &mut Func) -> String {
    fold(f);
    f.to_string()
}

#[test]
fn folds_a_constant_chain() {
    let mut f = Func::new("chain", 0);
    let three = f.const_id(3);
    let four = f.const_id(4);
    let two = f.const_id(2);
    let sum = f.bin(Op::Add, three, four);
    let prod = f.bin(Op::Mul, sum, two);
    f.set_ret(prod);

    assert_eq!(fold(&mut f), 2);
    assert_eq!(f.to_string(), "fn chain() -> i64 {\n  ret c14\n}");
}

#[test]
fn folds_through_a_removed_instruction() {
    // The `mul` is only foldable once `x + 0` collapses to `p0` and the mask
    // turns into the constant `0`.
    let mut f = Func::new("through", 1);
    let zero = f.const_id(0);
    let plus = f.bin(Op::Add, 0, zero);
    let masked = f.bin(Op::Mul, plus, zero);
    f.set_ret(masked);

    assert_eq!(fold(&mut f), 2);
    assert!(f.to_string().contains("ret c0"));
}

#[test]
fn drops_identities_and_keeps_the_parameter() {
    let mut f = Func::new("keep", 1);
    let zero = f.const_id(0);
    let one = f.const_id(1);
    let plus = f.bin(Op::Add, 0, zero);
    let times = f.bin(Op::Mul, one, plus);
    f.set_ret(times);

    assert_eq!(fold(&mut f), 2);
    assert_eq!(f.to_string(), "fn keep(%p0: i64) -> i64 {\n  ret %p0\n}");
}

#[test]
fn kills_self_referencing_ops() {
    let mut f = Func::new("self", 2);
    let and = f.bin(Op::And, 0, 0);
    let xor = f.bin(Op::Xor, 1, 1);
    let out = f.bin(Op::Add, and, xor);
    f.set_ret(out);

    assert_eq!(fold(&mut f), 3);
    let text = f.to_string();
    assert_eq!(text, "fn self(%p0: i64, %p1: i64) -> i64 {\n  ret %p0\n}");
}

#[test]
fn leaves_an_overflowing_add_alone() {
    let mut f = Func::new("wide", 0);
    let max = f.const_id(i64::MAX);
    let one = f.const_id(1);
    let add = f.bin(Op::Add, max, one);
    f.set_ret(add);

    assert_eq!(fold(&mut f), 0);
    assert!(f.to_string().contains("add"));
}

#[test]
fn is_idempotent() {
    let mut f = Func::new("twice", 2);
    let three = f.const_id(3);
    let mul = f.bin(Op::Mul, 0, three);
    let zero = f.const_id(0);
    let add = f.bin(Op::Add, mul, zero);
    f.set_ret(add);

    let once = show(&mut f);
    assert_eq!(fold(&mut f), 0);
    assert_eq!(f.to_string(), once);
}
