default: run

.PHONY: run
run:
		cargo run

.PHONY: disasm-vim
disasm-vim:
		cargo disasm > /tmp/os-disasm.txt;vim /tmp/os-disasm.txt;rm /tmp/os-disasm.txt
