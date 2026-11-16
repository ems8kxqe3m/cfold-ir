//! Prints one function before and after folding. `cargo run` is the quickest way
//! to see what the pass does to a small piece of IR.

use cfold_ir::ir::{Func, Op};
use cfold_ir::passes::fold::fold;

/// `fn checksum(p0, p1) -> i64 { (((p0 * 2) + 0) & 255) | (7 + 35) }`
fn demo() -> Func {
    let mut f = Func::new("checksum", 2);
    let two = f.const_id(2);
    let zero = f.const_id(0);
    let mask = f.const_id(255);
    let seven = f.const_id(7);
    let thirty_five = f.const_id(35);

    let scaled = f.bin(Op::Mul, 0, two);
    let shifted = f.bin(Op::Shl, 1, zero);
    let biased = f.bin(Op::Add, scaled, zero);
    let masked = f.bin(Op::And, biased, mask);
    let sum = f.bin(Op::Add, seven, thirty_five);
    let out = f.bin(Op::Or, masked, sum);
    let _ = shifted;
    f.set_ret(out);
    f
}

fn main() {
    let mut func = demo();
    println!("before:\n{func}\n");
    let removed = fold(&mut func);
    println!("after ({removed} instruction(s) folded):\n{func}");
}
