How to compile:
```sh
cargo build --release --color always --locked --target ../tuples/x86_64-unknown-kernel.json -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem
```

See objdump of binary
```sh
objdump ../target/x86_64-unknown-kernel/release/kernel -d
```
