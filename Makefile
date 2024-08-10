BOOTLOADER_SRC:=./src/boot.s
TARGET_DIR:=./target
TARGET:=${TARGET_DIR}/disk.img

all: ${TARGET}

run: all
	qemu-system-x86_64 -drive file=${TARGET},format=raw,index=0,media=disk

test: all
	qemu-system-x86_64 -s -S -drive file=${TARGET},format=raw,index=0,media=disk &
	gdb

clean:
	rm -rf ${TARGET_DIR}

${TARGET}: rust_src
	objcopy -I elf32-i386 -O binary target/i386-code16-boot-sector/release/spenceros2 ${TARGET}

rust_src : ${BOOTLOADER_SRC} $(find src/ -type f -name "*.rs")
	cargo build -r --target=./tuples/i386-code16-boot-sector.json

target_dir:
	@mkdir -p ${TARGET_DIR}

