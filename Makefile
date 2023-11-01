ARCH ?= aarch64
MACHINE ?= qemu
PROFILE ?= release

TOOLCHAIN ?=

LOG ?= info

BUS ?= mmio

APP ?= hello_world

APP_BIN ?= ${APP}

ifneq ($(findstring $(MACHINE), qemu shyper),) # if findstring not null
LD_SCRIPT := $(CURDIR)/cfg/$(ARCH)linker.ld
else
LD_SCRIPT := $(CURDIR)/cfg/$(ARCH)linker-$(MACHINE).ld
endif

export MACHINE
export ARCH
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
	@for dir in ${EXAMPLE_DIRS}; do rm ./$$dir/*.elf ./$$dir/*.bin ./$$dir/*.asm 2> /dev/null || true; done
	@echo clean project done!

run: build
ifeq ($(MACHINE), qemu)
	$(call qemu_run)
endif
# Call mkimage to build image for TX2 booting.
ifeq ($(MACHINE), tx2)
	@echo "Build for run on Nvidia Tegra X2, binary file at ${OUT_BIN}, calling mkimage..."
	mkimage -n unishyper -A arm64 -O linux -C none -T kernel -a 0x80080000 -e 0x80080000 -d ${OUT_BIN} ${OUT_APP}.ubi
	cp ${OUT_ELF} /tftp/$(APP_BIN)_${TARGET_DESC}_${PROFILE}
	cp ${OUT_APP}.ubi /tftp/$(APP_BIN)_${TARGET_DESC}_${PROFILE}.ubi
	@echo "tftp 0xc0000000 ${TFTP_IPADDR}:$(APP_BIN)_${TARGET_DESC}_${PROFILE}; tftp 0x8a000000 ${TFTP_IPADDR}:$(APP_BIN)_${TARGET_DESC}_${PROFILE}.ubi; bootm start 0x8a000000 - 0x80000000; bootm loados; bootm go"
endif
ifeq ($(MACHINE), rk3588)
	@echo "Build for run on firefly roc-rk3588s-pc, binary file at ${OUT_BIN}, calling mkimage..."
	mkimage -n unishyper -A arm64 -O linux -C none -T kernel -a 0x00400000 -e 0x00400000 -d ${OUT_BIN} ${OUT_APP}.ubi
	scp ${OUT_ELF} tx2@192.168.106.153:/tftp/$(APP_BIN)_${TARGET_DESC}_${PROFILE}
	scp ${OUT_APP}.ubi tx2@192.168.106.153:/tftp/$(APP_BIN)_${TARGET_DESC}_${PROFILE}.ubi
	@echo "tftp 0x80000000 192.168.106.153:$(APP_BIN)_${TARGET_DESC}_${PROFILE}; tftp 0x00400000 192.168.106.153:$(APP_BIN)_${TARGET_DESC}_${PROFILE}.ubi;tftp 0x10000000 192.168.106.153:rk3588.bin; bootm 0x00400000 - 0x10000000;"
endif
ifeq ($(MACHINE), shyper)
	@echo "Build for run on Shyper Hypervisor, binary file at ${OUT_BIN}."
endif
ifeq ($(MACHINE), k210)
	@echo "Build for run on Kendryte K210. binary file at ${OUT_BIN}, "
ifneq ($(wildcard rustsbi-k210.bin),)
	cat rustsbi-k210.bin ${OUT_BIN} > ${OUT_APP}-flash.bin
	@echo "Calling kflash..."
# Flash to k210 on /dev/ttyUSB0 by kflash
	sudo kflash -tp /dev/ttyUSB0 -b 3000000 -B dan ${OUT_APP}-flash.bin
else
	@echo "Download RustSBI from https://github.com/rustsbi/rustsbi/releases/tag/v0.1.1"
# rustsbi:
# 	wget https://github.com/rustsbi/rustsbi/releases/download/v0.1.1/rustsbi-k210.zip | unzip rustsbi-k210.zip
# 	# bundle with offset at 0x20000
# 	truncate --size=128K rustsbi-k210.bin
endif
endif

debug: build
	$(call qemu_debug)

gdb:
	$(GDB) $(OUT_ELF) \
	  -ex 'target remote localhost:1234' \
	  -ex 'disp /16i $$pc' \
	  -ex 'set print asm-demangle on' \

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
