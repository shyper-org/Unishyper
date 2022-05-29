ARCH ?= aarch64
MACHINE ?= shyper
PROFILE ?= release
USER_PROFILE ?= release
TRUSTED_PROFILE ?= release

# NOTE: this is to deal with `(signal: 11, SIGSEGV: invalid memory reference)`
# https://github.com/rust-lang/rust/issues/73677
RUSTFLAGS := -C llvm-args=-global-isel=false

# NOTE: generate frame pointer for every function
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

CARGO_FLAGS := ${CARGO_FLAGS} #--features ${MACHINE}
CARGO_FLAGS := ${CARGO_FLAGS} --release


KERNEL := target/${ARCH}${MACHINE}/${PROFILE}/rust_shyper_os
USER_KERNEL := user/target/${ARCH}${MACHINE}/${PROFILE}/user

.PHONY: all build emu debug clean user

user:
	make -C user

build: 
	cargo build --lib --target ${ARCH}${MACHINE}.json -Z build-std=core,alloc  ${CARGO_FLAGS}

# aarch64-elf-objcopy ${KERNEL} -O binary ${KERNEL}.bin
# aarch64-elf-objdump --demangle -d ${KERNEL} > ${KERNEL}.asm

clean:
	-cargo clean
	make -C user clean

QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53 -device loader,file=${KERNEL},addr=0x80000000,force-raw=on
QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} -kernel ${KERNEL}.bin -s

user_emu: user
	qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${USER_KERNEL},addr=0x80000000,force-raw=on \
		-serial stdio -display none \
		-smp 4 -m 2048 \
		-kernel ${USER_KERNEL}.bin -s

debug: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} -kernel ${KERNEL}.bin -s -S

user_debug: user
	qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${USER_KERNEL},addr=0x80000000,force-raw=on \
		-serial stdio -display none \
		-smp 4 -m 2048 \
		-kernel ${USER_KERNEL}.bin -s -S

dependencies:
	rustup component add rust-src