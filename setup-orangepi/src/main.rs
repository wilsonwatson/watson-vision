fn main() {
    assert!(std::process::Command::new("cargo")
        .args(["build", "--release", "-p", "watson-vision"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success());

    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
    {
        std::fs::copy("target/release/watson-vision", "/usr/bin/watson-vision").unwrap();
        create_systemd();
    }
}

#[allow(dead_code)]
fn create_systemd() {
    std::fs::write(
        "/etc/systemd/system/watson-vision.service",
        r#"[Unit]
Description=Watson Vision

[Service]
ExecStart=/usr/bin/watson-vision

[Install]
WantedBy=multi-user.target
"#,
    )
    .unwrap();
    assert!(std::process::Command::new("systemctl")
        .args(["daemon-reload"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success());
    assert!(std::process::Command::new("systemctl")
        .args(["enable", "watson-vision.service"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap()
        .success());
}
