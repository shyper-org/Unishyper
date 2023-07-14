ARCH ?= aarch64
MACHINE ?= qemu
PROFILE ?= release

# Panic Inject Function
export PI

# NOTE: this is to deal with `(signal: 11, SIGSEGV: invalid memory reference)`
# https://github.com/rust-lang/rust/issues/73677
RUSTFLAGS := -C llvm-args=-global-isel=false

# NOTE: generate frame pointer for every function
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

CARGO_FLAGS := ${CARGO_FLAGS} #--features ${MACHINE}
CARGO_FLAGS := ${CARGO_FLAGS} --release

EXAMPLES_DIR := $(shell find examples -maxdepth 1 -mindepth 1 -type d)

USER_DIR := examples/user
FS_DEMO_DIR := examples/fs_demo
NET_DEMO_DIR := examples/net_demo
HTTPD_DIR := examples/httpd
LINUX_TEST_DIR := examples/linux_test

.PHONY: all build clean user net_bw_server net_bw_client disk tap_setup linux_test

build: 
	cargo build --lib --target ./cfg/${ARCH}${MACHINE}.json -Z build-std=core,alloc  ${CARGO_FLAGS}

clean:
	-cargo clean
	@for dir in ${EXAMPLES_DIR}; do make -C ./$$dir clean; done
	@echo clean project done!

# rust-objcopy ${KERNEL} -O binary ${KERNEL}.bin
# rust-objdump --demangle -d ${KERNEL} > ${KERNEL}.asm
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

fs:
	make -C ${FS_DEMO_DIR} emu

net:
	make -C ${NET_DEMO_DIR} emu

httpd:
	make -C ${HTTPD_DIR} emu

user_debug:
	make -C ${USER_DIR} debug

fs_debug:
	make -C ${FS_DEMO_DIR} debug

net_debug:
	make -C ${NET_DEMO_DIR} debug

linux_test:
	make -C ${LINUX_TEST_DIR} build

tx2:
	MACHINE=tx2 make -C ${USER_DIR} tx2

shyper: 
	MACHINE=shyper make -C ${USER_DIR} shyper

disk:
	rm -rf disk
	dd if=/dev/zero of=disk.img bs=4096 count=92160 2>/dev/null
	mkfs.fat -F 32 disk.img

# Setup tap0 device before run network in qemu.
tap_setup:
	sudo ip tuntap add tap0 mode tap
	sudo ip addr add 10.0.0.1/24 broadcast 10.0.0.255 dev tap0
	sudo ip link set dev tap0 up
	sudo bash -c 'echo 1 > /proc/sys/net/ipv4/conf/tap0/proxy_arp'

dependencies:
	rustup component add rust-src
