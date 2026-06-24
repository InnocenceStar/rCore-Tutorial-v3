# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000
QEMU_MEMORY_SIZE := 128M
QEMU_RAM_BASE_PA := 0x80000000
TRACE_BASE_PA := 0x86d00000
TRACE_RAM_IMG ?= target/$(TARGET)/$(MODE)/qemu-ram.img
TRACE_BYTRACE ?= target/$(TARGET)/$(MODE)/trace.bytrace
OSTRACE_GIT := https://github.com/rcore-os/ostrace.git
OSTRACE_REV := 9550ec9418dcfc1bfef1efa19bfae6d53649173b
OSTRACE_TOOL_ROOT := target/host-tools/ostrace
TRACE2BYTRACE := $(OSTRACE_TOOL_ROOT)/bin/trace2bytrace
HOST_TARGET := $(shell rustc -vV | sed -n 's/^host: //p')

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x

# Run usertests or usershell
TEST ?=

build: env $(KERNEL_BIN)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

kernel:
	@cd ../user && make build TEST=$(TEST)
	@echo Platform: $(BOARD)
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build --release
	@rm src/linker.ld

clean:
	@cargo clean

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

run: run-inner

QEMU_ARGS := -object memory-backend-file,id=trace-ram,size=$(QEMU_MEMORY_SIZE),mem-path=$(TRACE_RAM_IMG),share=on \
			 -machine virt,memory-backend=trace-ram \
			 -nographic \
			 -bios $(BOOTLOADER) \
			 -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

QEMU_NAME := qemu-system-riscv64
qemu-version-check:
	@sh scripts/qemu-ver-check.sh $(QEMU_NAME)

run-inner: qemu-version-check build
	@mkdir -p $(dir $(TRACE_RAM_IMG))
	@rm -f $(TRACE_RAM_IMG) $(TRACE_BYTRACE)
	@qemu-system-riscv64 $(QEMU_ARGS)
	@$(MAKE) trace-bytrace

debug: qemu-version-check build
	@mkdir -p $(dir $(TRACE_RAM_IMG))
	@rm -f $(TRACE_RAM_IMG)
	@tmux new-session -d \
		"qemu-system-riscv64 $(QEMU_ARGS) -s -S" && \
		tmux split-window -h "riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d

gdbserver: qemu-version-check build
	@mkdir -p $(dir $(TRACE_RAM_IMG))
	@rm -f $(TRACE_RAM_IMG)
	@qemu-system-riscv64 $(QEMU_ARGS) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

trace-bytrace:
	@cargo install --git $(OSTRACE_GIT) --rev $(OSTRACE_REV) --target $(HOST_TARGET) --features std --bin trace2bytrace --root $(OSTRACE_TOOL_ROOT)
	@$(TRACE2BYTRACE) \
		--ram $(TRACE_RAM_IMG) \
		--out $(TRACE_BYTRACE) \
		--ram-base $(QEMU_RAM_BASE_PA) \
		--trace-base $(TRACE_BASE_PA)
	@echo Trace bytrace: $(TRACE_BYTRACE)

.PHONY: build env kernel clean disasm disasm-vim run-inner gdbserver gdbclient qemu-version-check trace-bytrace
