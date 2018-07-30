use std::process::Command;

fn main() {
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
}
