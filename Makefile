ARCH ?= aarch64
MACHINE ?= qemu
PROFILE ?= release

CARGO_TOOLCHAIN ?= 

LOG ?= info

APP ?= user

APP_BIN ?= ${APP}

# Panic Inject Function
export PI

EXAMPLE_DIRS := $(shell find examples -maxdepth 1 -mindepth 1 -type d)

ifeq ($(wildcard examples/${APP}),)
  	$(error Dir "examples/${APP}" not exist, existing examples contain [${EXAMPLE_DIRS}], or you may create your own example using "cargo new --PROJECT_NAME")
endif

include scripts/build.mk
include scripts/qemu.mk

.PHONY: all bootloader build clean run debug gdb disk tap_setup

all: build

bootloader:
ifeq ($(ARCH), x86_64)
	cd ${RBOOT_DIR} && make build
endif

build:
	$(call cargo_build)
ifeq ($(ARCH), x86_64)
	$(call rboot_pre)
endif

clean:
	-cargo clean
	@for dir in ${EXAMPLE_DIRS}; do make -C ./$$dir clean; done
	@echo clean project done!

run: build
	$(call qemu_run)

debug: build
	$(call qemu_debug)

gdb:
	$(GDB) $(OUT_ELF) \
	  -ex 'target remote localhost:1234' \
	  -ex 'b _start' \
	  -ex 'continue' \
	  -ex 'disp /16i $$pc' \
	  -ex 'set print asm-demangle on'

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
