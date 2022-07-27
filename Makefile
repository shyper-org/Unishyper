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

USER_DIR := examples/user
NET_DEMO_DIR := examples/net_demo

.PHONY: all build clean user net_server net_client disk tap_setup net_server_debug net_client_debug

build: 
	cargo build --lib --target ${ARCH}${MACHINE}.json -Z build-std=core,alloc  ${CARGO_FLAGS}

clean:
	-cargo clean
	make -C examples/user clean
	make -C ${NET_DEMO_DIR} clean

# aarch64-elf-objcopy ${KERNEL} -O binary ${KERNEL}.bin
# aarch64-elf-objdump --demangle -d ${KERNEL} > ${KERNEL}.asm
# QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53 -device loader,file=${KERNEL},addr=0x80000000,force-raw=on
# QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
# 					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
# 					 -global virtio-mmio.force-legacy=false
# QEMU_NETWORK_OPTIONS := -netdev tap,id=tap0,ifname=tap0,script=no,downscript=no \
# 						-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=tap0 \
# 						-global virtio-mmio.force-legacy=false
# QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

user:
	make -C ${USER_DIR} emu

net_server:
	make -C ${NET_DEMO_DIR} server_emu

net_client:
	make -C ${NET_DEMO_DIR} client_emu

net_server_debug:
	make -C ${NET_DEMO_DIR} server_debug

net_client_debug:
	make -C ${NET_DEMO_DIR} client_debug

disk:
	rm -rf disk
	dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
	mkfs.fat -F 32 disk.img

tap_setup:
	sudo ip tuntap add tap0 mode tap
	sudo ip addr add 10.0.0.1/24 broadcast 10.0.0.255 dev tap0
	sudo ip link set dev tap0 up
	sudo bash -c 'echo 1 > /proc/sys/net/ipv4/conf/tap0/proxy_arp'

dependencies:
	rustup component add rust-src
