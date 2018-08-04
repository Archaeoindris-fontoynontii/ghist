use std::fs;
use std::process::Command;

fn main() -> std::io::Result<()> {
    let p = Command::new("npm")
        .current_dir("static")
        .arg("run")
        .arg(if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        })
        .output();
    println!("{:?}", p);
    println!("cargo:rerun-if-changed={}", "static/index.html");
    println!("cargo:rerun-if-changed={}", "static/style.css");
    for entry in fs::read_dir("static/src")? {
        println!("cargo:rerun-if-changed={}", entry?.path().display());
    }
    Ok(())
}
