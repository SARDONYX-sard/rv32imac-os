FROM rustlang/rust:nightly-alpine

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

# musl-dev: for cargo-binutils(https://stackoverflow.com/questions/6329887/how-to-fix-linker-error-cannot-find-crt1-o)
# qemu: https://wiki.alpinelinux.org/wiki/Install_Alpine_in_QEMU
# hadolint ignore=DL3018
RUN apk add --no-cache \
    git \
    gnupg \
    make \
    musl-dev \
    tmux \
    qemu-system-riscv32 \
    vim && \
    rustup target add riscv32imac-unknown-none-elf && \
    cargo install cargo-binutils && \
    rustup component add clippy rustfmt rust-src llvm-tools

WORKDIR /mnt/app
