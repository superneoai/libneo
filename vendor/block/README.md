# Vendored block 0.1.6

## Provenance

This directory comes from the crates.io release [`block 0.1.6`]. The source
archive has this SHA-256 digest:

```text
0d8c1fef690941d3e7788d328517591fecc684c084084702d6ff1641e993699a  block-0.1.6.crate
```

Apart from this provenance section and Markdown-only cleanup of the upstream
README, the local copy changes only the following:

- uses an inhabited opaque marker for the external block class;
- gives foreign functions and callbacks explicit C ABIs;
- removes the unavailable upstream test helper and uses self-contained tests;
- declares the original Rust 2015 edition explicitly;
- sets `publish = false`;
- includes the upstream MIT license text.

To refresh and verify the source:

```sh
curl --proto '=https' --tlsv1.2 -fL \
  -o block-0.1.6.crate \
  https://static.crates.io/crates/block/block-0.1.6.crate
printf '%s  %s\n' \
  0d8c1fef690941d3e7788d328517591fecc684c084084702d6ff1641e993699a \
  block-0.1.6.crate | shasum -a 256 -c -
tar -xzf block-0.1.6.crate
cargo test --manifest-path vendor/block/Cargo.toml
```

Compare the unpacked `block-0.1.6` directory with `vendor/block`, then reapply
only the modifications listed above.

[`block 0.1.6`]: https://crates.io/crates/block/0.1.6

## Upstream README

Rust interface for Apple's C language extension of blocks.

For more information on the specifics of the block implementation, see
[Clang's Block ABI documentation].

[Clang's Block ABI documentation]: https://clang.llvm.org/docs/Block-ABI-Apple.html

### Invoking blocks

The `Block` struct is used for invoking blocks from Objective-C. For example,
consider this Objective-C function:

```objc
int32_t sum(int32_t (^block)(int32_t, int32_t)) {
    return block(5, 8);
}
```

We could write it in Rust as the following:

```rust
unsafe fn sum(block: &Block<(i32, i32), i32>) -> i32 {
    block.call((5, 8))
}
```

Note the extra parentheses in the `call` method, since the arguments must be
passed as a tuple.

### Creating blocks

Creating a block to pass to Objective-C can be done with the `ConcreteBlock`
struct. For example, to create a block that adds two `i32`s, we could write:

```rust
let block = ConcreteBlock::new(|a: i32, b: i32| a + b);
let block = block.copy();
assert!(unsafe { block.call((5, 8)) } == 13);
```

It is important to copy your block to the heap (with the `copy` method) before
passing it to Objective-C; this is because our `ConcreteBlock` is only meant
to be copied once, and we can enforce this in Rust, but if Objective-C code
were to copy it twice we could have a double free.
