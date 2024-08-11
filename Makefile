TARGET_DIR:=./target
TARGET:=${TARGET_DIR}/disk.img

all: cargo

run: all
	qemu-system-x86_64 -drive file=${TARGET},format=raw,index=0,media=disk

test: all
	qemu-system-x86_64 -s -S -drive file=${TARGET},format=raw,index=0,media=disk &
	gdb

clean:
	rm -rf ${TARGET_DIR}

# Cargo internally does checking on changes in the build.rs build scripts.
cargo:
	cargo run 
