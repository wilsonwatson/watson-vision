sudo apt update && TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt install libopencv-dev clang libclang-dev curl build-essential -y
curl https://sh.rustup.rs -sSf | bash -s -- -y
$HOME/.cargo/bin/cargo run -p setup-orangepi