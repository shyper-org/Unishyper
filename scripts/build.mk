# Utils
OBJCOPY := rust-objcopy
OBJDUMP := rust-objdump

# Rust flags, for unwind.
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

# Cargo flags.
ifeq (${PROFILE}, release)
CARGO_FLAGS = --release  --no-default-features
else
CARGO_FLAGS =  --no-default-features
endif

TARGET_CFG := $(CURDIR)/cfg/${ARCH}${MACHINE}.json

TARGET_DIR := $(CURDIR)/target

# Kernel directory.
KERNEL_DIR := ${ROOT_DIR}/target/${ARCH}${MACHINE}/${PROFILE}
APP_DIR := examples/$(APP)

OUT_DIR := ${TARGET_DIR}/${ARCH}${MACHINE}/${PROFILE}
BUILD_ELF := ${OUT_DIR}/${APP_BIN}

OUT_APP := ${APP_DIR}/$(APP_BIN)_${ARCH}_${MACHINE}_${PROFILE}
OUT_ELF := ${OUT_APP}.elf
OUT_BIN := ${OUT_APP}.bin
OUT_ASM := ${OUT_APP}.asm

CARGO_ARGS := \
	--manifest-path ${APP_DIR}/Cargo.toml \
	--bin ${APP_BIN} \
	--target ${TARGET_CFG} \
	--target-dir ${TARGET_DIR} \
	-Z build-std=core,alloc \
	-Z build-std-features=compiler-builtins-mem \
	${CARGO_FLAGS}

ifeq ($(filter $(ARCH),aarch64 x86_64 riscv64),)
  $(error "ARCH" must be one of "aarch64", "x86_64", "riscv64")
endif

ifeq ($(filter $(LOG),off error warn info debug trace),)
  $(error "LOG" must be one of "off", "error", "warn", "info", "debug", "trace")
endif

FEATURES := ${MACHINE}, unishyper/log-level-${LOG}

# Currently we use [rboot](https://github.com/hky1999/rboot.git) as bootloader in x86_64.
ifeq ($(ARCH), x86_64)
RBOOT_DIR := $(CURDIR)/x86boot
OVMF := ${RBOOT_DIR}/OVMF.fd
endif

define rboot_pre
	cp $(OUT_ELF) $(RBOOT_DIR)/EFI/Demo/kernel.elf
endef

define cargo_build
	cargo build $(CARGO_ARGS) --features "${FEATURES}"
	@cp $(BUILD_ELF) $(OUT_ELF)
	${OBJCOPY} ${OUT_ELF} -O binary ${OUT_BIN}
	${OBJDUMP} --demangle -d ${OUT_ELF} > ${OUT_ASM}
endef

$(OUT_BIN):
