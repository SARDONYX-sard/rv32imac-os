use std::fs::{read_dir, File};
use std::io::{Result, Write};
use std::path::Path;

const TARGET_PATH: &str = "target/riscv32imac-unknown-none-elf/release/";
/// 64bit: quad, 32bit: word
/// https://github.com/riscv-non-isa/riscv-asm-manual/blob/master/riscv-asm.md#pseudo-ops
const PTR_SIZE: &str = "word";

fn main() {
    let user_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!("cargo:rerun-if-changed={}", user_dir.display());

    // set user link file
    println!(
        "cargo:rustc-link-arg=-T{}",
        user_dir.join("user.ld").display()
    );

    // println!("cargo:rerun-if-changed={}", TARGET_PATH);
    create_app_data(user_dir).unwrap();
}

fn create_app_data(manifest_dir: &Path) -> Result<()> {
    let mut f = File::create(manifest_dir.join("link_app.S")).unwrap();
    println!("{:?}", manifest_dir.join("bin"));
    let mut apps: Vec<_> = read_dir(manifest_dir.join("bin"))
        .unwrap()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    writeln!(
        f,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .{PTR_SIZE} {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        writeln!(f, r#"    .{PTR_SIZE} app_{}_start"#, i)?;
    }
    writeln!(f, r#"    .{PTR_SIZE} app_{}_end"#, apps.len() - 1)?;

    for (idx, app) in apps.iter().enumerate() {
        println!("app_{}: {}", idx, app);

        writeln!(
            f,
            r#"
    .section .data
    .global app_{0}_start
    .global app_{0}_end
app_{0}_start:
    .incbin "{2}{1}.bin"
app_{0}_end:"#,
            idx, app, TARGET_PATH
        )?;
    }
    Ok(())
}
