use cfold_ir::ir::{Func, Op};
use cfold_ir::passes::fold::fold;
use cfold_ir::passes::strength::reduce;

#[test]
fn rewrites_multiply_by_power_of_two() {
    let mut f = Func::new("scale", 1);
    let eight = f.const_id(8);
    let scaled = f.bin(Op::Mul, 0, eight);
    f.set_ret(scaled);

    assert_eq!(reduce(&mut f), 1);
    assert_eq!(
        f.to_string(),
        "fn scale(%p0: i64) -> i64 {\n  %v2 = shl %p0, c3\n  ret %v2\n}"
    );
}

#[test]
fn rewrites_x_plus_x() {
    let mut f = Func::new("double", 1);
    let doubled = f.bin(Op::Add, 0, 0);
    f.set_ret(doubled);

    assert_eq!(reduce(&mut f), 1);
    assert_eq!(
        f.to_string(),
        "fn double(%p0: i64) -> i64 {\n  %v1 = shl %p0, c1\n  ret %v1\n}"
    );
}

#[test]
fn leaves_other_multipliers_alone() {
    let mut f = Func::new("three", 1);
    let three = f.const_id(3);
    let scaled = f.bin(Op::Mul, 0, three);
    f.set_ret(scaled);

    assert_eq!(reduce(&mut f), 0);
    assert!(f.to_string().contains("mul"));
}

#[test]
fn composes_with_folding() {
    // `p0 + 0` has to fold away before `* 4` can be seen as a shift by two.
    let mut f = Func::new("compose", 1);
    let zero = f.const_id(0);
    let four = f.const_id(4);
    let plus = f.bin(Op::Add, 0, zero);
    let scaled = f.bin(Op::Mul, plus, four);
    f.set_ret(scaled);

    assert_eq!(fold(&mut f), 1);
    assert_eq!(reduce(&mut f), 1);
    assert_eq!(
        f.to_string(),
        "fn compose(%p0: i64) -> i64 {\n  %v4 = shl %p0, c2\n  ret %v4\n}"
    );
}
