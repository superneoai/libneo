# Contributing

## License

libneo uses `MIT OR Apache-2.0`. A contribution enters under the same terms.

## Checks

See the [README build section](README.md#build) for the complete check list.

## Style

- Write comments and documentation in concise, plain English.
- Use `//!` for crate and module documentation and `///` for public items.
- Explain purpose, invariants, and safety requirements rather than restating
  the code.
- Keep facades dependency-neutral and place adapter code in its focused crate.
- Keep GPUI native code in `crates/libneo-gpui/src/platform`.
