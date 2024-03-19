sudo apt install libopencv-dev clang libclang-dev -y
curl https://sh.rustup.rs -sSf | bash -s -- -y
echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
source $HOME/.bashrc
cargo run -p setup-orangepi