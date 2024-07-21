# matrix

![matrix](https://github.com/jiaoshijie/matrix/assets/43605101/bf3bf017-f4d2-45cd-a1b6-76514be787fe)

## cross compile from x86_64 linux gnu-libc to aarch64 linux musl-lic

- install `aarch64-linux-gnu-gcc` compiler
- `rustup component add rust-std-aarch64-unknown-linux-musl`
- `export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc`
- `export CC=aarch64-linux-gnu-gcc`
- `cargo build --release --target=aarch64-unknown-linux-musl`

## Ref

- [cross compile error](https://github.com/rust-lang/stacker/issues/80)
- [target triple explaination](https://clang.llvm.org/docs/CrossCompilation.html)
