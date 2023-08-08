default: run

.PHONY: run
run:
		cargo run

.PHONY: disasm-vim
disasm-vim:
		cargo dmp > ./dump.txt
