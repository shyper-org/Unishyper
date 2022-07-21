ARCH ?= aarch64
MACHINE ?= shyper
PROFILE ?= release

# NOTE: this is to deal with `(signal: 11, SIGSEGV: invalid memory reference)`
# https://github.com/rust-lang/rust/issues/73677
RUSTFLAGS := -C llvm-args=-global-isel=false

# NOTE: generate frame pointer for every function
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

CARGO_FLAGS := ${CARGO_FLAGS} #--features ${MACHINE}
CARGO_FLAGS := ${CARGO_FLAGS} --release


KERNEL := target/${ARCH}${MACHINE}/${PROFILE}/rust_shyper_os
USER_KERNEL := examples/user/target/${ARCH}${MACHINE}/${PROFILE}/user
NET_KERNEL := examples/net_demo/target/${ARCH}${MACHINE}/${PROFILE}/net_demo

.PHONY: all build emu debug clean user net_demo user_emu net_emu disk

user:
	make -C examples/user

net_demo:
	make -C examples/net_demo

build: 
	cargo build --lib --target ${ARCH}${MACHINE}.json -Z build-std=core,alloc  ${CARGO_FLAGS}

# aarch64-elf-objcopy ${KERNEL} -O binary ${KERNEL}.bin
# aarch64-elf-objdump --demangle -d ${KERNEL} > ${KERNEL}.asm

clean:
	-cargo clean
	make -C examples/user clean
	make -C examples/net_demo clean

QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53 -device loader,file=${KERNEL},addr=0x80000000,force-raw=on
QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap10,script=no,downscript=no \
						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
						-global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} -kernel ${KERNEL}.bin -s

debug: build
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} -kernel ${KERNEL}.bin -s -S

user_emu: user
	qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${USER_KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_DISK_OPTIONS} \
		-kernel ${USER_KERNEL}.bin -s

net_emu: net_demo
	sudo qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${NET_KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		${QEMU_DISK_OPTIONS} \
		-kernel ${NET_KERNEL}.bin -s

user_debug: user
	qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${USER_KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${USER_KERNEL}.bin -s -S

net_debug: net_demo
	qemu-system-aarch64 -M virt -cpu cortex-a53 \
		-device loader,file=${NET_KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_COMMON_OPTIONS} \
		${QEMU_NETWORK_OPTIONS} \
		-kernel ${NET_KERNEL}.bin -s -S

disk:
	rm -rf disk
	dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
	mkfs.fat -F 32 disk.img

tap_setup:
	sudo ip tuntap add tap10 mode tap
	sudo ip addr add 10.0.5.1/24 broadcast 10.0.5.255 dev tap10
	sudo ip link set dev tap10 up
	sudo bash -c 'echo 1 > /proc/sys/net/ipv4/conf/tap10/proxy_arp'

dependencies:
	rustup component add rust-src
