use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let user_src_path = build_user_bins();
    println!("cargo:rerun-if-changed={}", user_src_path.display());

    println!("cargo:rustc-link-arg-bin=os=--script=src/kernel/kernel.ld");
}

fn build_user_bins() -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg("OS1000lineUser");
    let user_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("user");
    if user_path.exists() {
        // local build
        cmd.arg("--path").arg(&user_path);
        println!("cargo:rerun-if-changed={}", user_path.display());
    }

    let status = cmd
        .status()
        .expect("failed to run cargo install for user_bins");
    if status.success() {
        let mut objcopy = Objcopy::new();

        let bins = std::fs::read_dir("target/riscv32imac-unknown-none-elf/release/")
            .unwrap()
            .filter(|path| {
                path.as_ref()
                    .unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .starts_with('_')
            })
            .map(|path| path.as_ref().unwrap().path());
        for bin in bins.into_iter() {
            if bin.extension().is_none() {
                objcopy.run(bin);
            };
        }

        user_path
    } else {
        panic!("failed to build user programs");
    }
}

struct Objcopy {
    cmd: Command,
}

impl Objcopy {
    pub fn new() -> Self {
        let mut objcopy = Command::new("rust-objcopy");
        objcopy
            .arg("--binary-architecture=riscv32")
            .arg("--strip-all")
            .args(["-O", "binary"]);

        Self { cmd: objcopy }
    }

    pub fn run(&mut self, bin_name: PathBuf) {
        let dist = {
            let mut dist = bin_name.clone();
            dist.set_extension("bin");
            dist
        };
        let src = bin_name;
        let status = self.cmd.arg(src).arg(dist).status().unwrap();
        assert!(status.success());
    }
}
