all: build

run: test
	cargo run

build:
	cargo build


TEST_FLAGS:=--release --color always --locked  -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem
test: 
	cargo test --package common
	cargo build --package common --target ./tuples/i386-bit16.json ${TEST_FLAGS}
	cargo build --package common --target ./tuples/i386-bit32.json ${TEST_FLAGS}
	cargo build --package common --target ./tuples/x86_64-unknown-kernel.json ${TEST_FLAGS}
    # let target_file: PathBuf = format!("./tuples/{target}.json").into();
	cargo build --package stage-0 --target ./tuples/i386-bit16.json ${TEST_FLAGS}
	cargo build --package stage-1 --target ./tuples/i386-bit16.json ${TEST_FLAGS}
	cargo build --package stage-2 --target ./tuples/i386-bit32.json ${TEST_FLAGS}
	cargo build --package stage-3 --target ./tuples/x86_64-unknown-kernel.json ${TEST_FLAGS}
	cargo test
