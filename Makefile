default: run

.PHONY: run
run:
		cargo run

.PHONY: disasm-vim
disasm-vim:
		cargo dmp > ./dump.txt

.PHONY: shell-dump
shell-dump:
		rust-objdump ./target/riscv32imac-unknown-none-elf/release/_shell --disassemble-all --arch-name=riscv32
