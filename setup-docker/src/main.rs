fn main() {
    assert!(std::process::Command::new("cargo")
        .args(["build", "--release", "-p", "watson-vision"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success());
    std::fs::copy("target/release/watson-vision", "/usr/bin/watson-vision").unwrap();
}
