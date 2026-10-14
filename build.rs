use std::{env, path::PathBuf, process::Command};

fn run(mut command: Command) {
    let status = command
        .status()
        .expect("No se pudo ejecutar la herramienta de BlocksDS");
    assert!(status.success(), "Falló {command:?}");
}

fn main() {
    println!("cargo:rerun-if-changed=native/nds.c");
    for key in ["BLOCKSDS", "WONDERFUL_TOOLCHAIN", "DEVY_CC", "DEVY_AR"] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    if env::var("CARGO_CFG_TARGET_VENDOR").as_deref() != Ok("nintendo") {
        return;
    }
    assert_eq!(
        env::var("TARGET").as_deref(),
        Ok("armv5te-nintendo-ds"),
        "Usa el target JSON de devy"
    );
    let sdk = PathBuf::from(env::var_os("BLOCKSDS").expect("Define BLOCKSDS con la ruta del SDK"));
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let toolchain = PathBuf::from(
        env::var_os("WONDERFUL_TOOLCHAIN").unwrap_or_else(|| "/opt/wonderful".into()),
    );
    let bin = toolchain.join("toolchain/gcc-arm-none-eabi/bin");
    let cc = env::var_os("DEVY_CC").unwrap_or_else(|| bin.join("arm-none-eabi-gcc").into());
    let ar = env::var_os("DEVY_AR").unwrap_or_else(|| bin.join("arm-none-eabi-ar").into());
    let specs = sdk.join("sys/crts/ds_arm9.specs");
    assert!(specs.is_file(), "Falta {}", specs.display());
    println!(
        "cargo:rerun-if-changed={}",
        sdk.join("libs/libnds/include").display()
    );
    println!("cargo:rerun-if-changed={}", specs.display());
    let mut compile = Command::new(cc);
    compile
        .args(["-c", "native/nds.c", "-o"])
        .arg(out.join("nds.o"))
        .args([
            "-std=gnu11",
            "-O2",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-Wconversion",
            "-ffunction-sections",
            "-fdata-sections",
            "-mthumb",
            "-mcpu=arm946e-s+nofp",
        ])
        .arg(format!("-specs={}", specs.display()))
        .arg("-isystem")
        .arg(sdk.join("libs/libnds/include"));
    if env::var_os("CARGO_FEATURE_STORAGE").is_some() {
        compile.arg("-DDEVY_STORAGE");
    }
    run(compile);
    let mut archive = Command::new(ar);
    archive
        .arg("crs")
        .arg(out.join("libdevy_nds.a"))
        .arg(out.join("nds.o"));
    run(archive);
    // Un archivo estático sí se propaga al ejecutable consumidor del crate.
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=devy_nds");
}
