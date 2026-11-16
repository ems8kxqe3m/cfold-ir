# cfold-ir

Constant folding for a tiny typed SSA IR, used as a workbench for pass
correctness. The point is not the IR — it is the property every rewrite has to
keep: a value's uses are rewritten, the defining instruction disappears, and the
printed IR stays valid.

The IR is one block over `i64`, values are numbered, parameters come first and
constants live in an interned table. Instructions are in dominance order, so the
folder can run as an ordered sweep instead of a worklist.

## Before / after

`cargo run` builds one function and folds it. Before:

```text
fn checksum(%p0: i64, %p1: i64) -> i64 {
  %v7 = mul %p0, c2
  %v8 = shl %p1, c0
  %v9 = add %v7, c0
  %v10 = and %v9, c255
  %v11 = add c7, c35
  %v12 = or %v10, %v11
  ret %v12
}
```

After:

```text
fn checksum(%p0: i64, %p1: i64) -> i64 {
  %v7 = mul %p0, c2
  %v10 = and %v7, c255
  %v12 = or %v10, c42
  ret %v12
}
```

`add %v7, c0` collapses into its operand, `shl %p1, c0` drops out, and `7 + 35`
becomes `c42`. Ids are not renumbered: sparse ids make diffs against the expected
IR in `tests/fold.rs` readable, and nothing downstream assumes density.

Three properties the pass keeps deliberately:

- An identity that exposes a constant is followed by another sweep, so folding
  reaches a fixpoint rather than stopping one instruction short.
- `checked_add`/`checked_sub`/`checked_mul` failure leaves the instruction in
  place. Wrapping instead would need an explicit `add wrap` op first.
- Shifts are only folded for amounts in `0..64`; anything else stays for the
  backend.

## Install

```sh
git clone https://github.com/ems8kxqe3m/cfold-ir
cd cfold-ir
cargo build
```

Rust 1.70 or newer; there are no dependencies.

## Usage

```sh
cargo run          # fold the built-in demo function
cargo test         # golden-IR and idempotence tests
```

From code, build a `Func` and call the pass:

```rust
use cfold_ir::ir::{Func, Op};
use cfold_ir::passes::fold::fold;

let mut f = Func::new("add_zero", 1);
let zero = f.const_id(0);
let sum = f.bin(Op::Add, 0, zero);
f.set_ret(sum);

assert_eq!(fold(&mut f), 1);
assert!(f.to_string().contains("ret %p0"));
```

## Layout

- `src/ir.rs` — ops, instruction and function types, printer.
- `src/passes/fold.rs` — the folding pass and its identity rules.
- `src/main.rs` — demo driver.
- `tests/fold.rs` — golden IR, fixpoint and overflow cases.

## License

MIT, see `LICENSE`.
