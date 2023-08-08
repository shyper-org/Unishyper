QEMU_CMD := sudo qemu-system-$(ARCH)

## Set machine type.

ifeq ($(ARCH), aarch64)
QEMU_CMD := ${QEMU_CMD} -M virt -cpu cortex-a53
endif

ifeq ($(ARCH), riscv64)
QEMU_CMD := ${QEMU_CMD} -M virt -bios default
endif

ifeq ($(ARCH), x86_64)
ifeq ($(KVM), true)
QEMU_CMD := ${QEMU_CMD} -cpu host -enable-kvm
else
QEMU_CMD := ${QEMU_CMD} -cpu qemu64,apic,fsgsbase,fxsr,rdrand,rdtscp,xsave,xsaveopt,pku
endif
endif

QEMU_CMD :=  ${QEMU_CMD} -smp 1 -m 2048

ifeq ($(ARCH), aarch64)
QEMU_CMD := ${QEMU_CMD} -device loader,file=${OUT_ELF},addr=0x80000000,force-raw=on -kernel ${OUT_BIN}
endif

ifeq ($(ARCH), riscv64)
QEMU_CMD := ${QEMU_CMD} -device loader,file=$(OUT_ELF),addr=0xc0000000,force-raw=on -kernel ${OUT_BIN}
endif

ifeq ($(ARCH), x86_64)
QEMU_CMD := ${QEMU_CMD} \
			-drive if=pflash,format=raw,readonly=on,file=$(OVMF) \
			-drive format=raw,file=fat:rw:$(RBOOT_DIR) \
			-device isa-debug-exit,iobase=0xf4,iosize=0x04
endif

# QEMU_DISK_OPTIONS

DISK_IMG ?= disk.img

ifeq ($(FS), y)
ifeq ($(wildcard ${DISK_IMG}),)
  	$(error File "${DISK_IMG}" not exist, you may create your own image by run "make disk")
endif
QEMU_CMD := ${QEMU_CMD} \
			-drive file=${DISK_IMG},if=none,format=raw,id=x0 \
			-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
			-global virtio-mmio.force-legacy=false
endif

# QEMU_NETWORK_OPTIONS

TAP_IF ?= tap0

ifeq ($(NET), y)
ifneq ($(ARCH), x86_64)
QEMU_CMD := ${QEMU_CMD} \
			-netdev tap,id=${TAP_IF},ifname=${TAP_IF},script=no,downscript=no \
			-device virtio-net-device,mac=48:b0:2d:0e:6e:9e,netdev=${TAP_IF} \
			-global virtio-mmio.force-legacy=false
endif
ifeq ($(ARCH), x86_64)
QEMU_CMD := ${QEMU_CMD} \
			-netdev tap,id=${TAP_IF},ifname=${TAP_IF},script=no,downscript=no,vhost=on \
			-device virtio-net-pci,netdev=${TAP_IF},disable-legacy=on,mac=48:b0:2d:0e:6e:9e
endif
endif

QEMU_COMMON_OPTIONS := -serial stdio -display none -s

define qemu_run
  ${QEMU_CMD} ${QEMU_COMMON_OPTIONS}
endef

define qemu_debug
  ${QEMU_CMD} ${QEMU_COMMON_OPTIONS} -S
endef