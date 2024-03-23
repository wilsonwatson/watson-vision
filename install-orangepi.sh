sudo apt-get update
TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt-get install libopencv-dev clang libclang-dev curl build-essential libstdc++-12-dev -y
TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt-get install libavcodec-dev libavformat-dev libswscale-dev libv4l-dev libxvidcore-dev libx264-dev libtbbmalloc2 libtbb-dev libjpeg-dev libpng-dev libtiff-dev libdc1394-dev gfortran openexr libatlas-base-dev -y
TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt-get install libgstreamer1.0-dev -y
TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt-get install libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-base -y
TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt-get install libgstreamer-plugins-bad1.0-dev  gstreamer1.0-plugins-bad -y
TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt-get install gstreamer1.0-plugins-good -y
TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt-get install gstreamer1.0-plugins-ugly -y
TZ=Us/Denver DEBIAN_FRONTEND=noninteractive sudo apt-get install gstreamer1.0-libav gstreamer1.0-tools gstreamer1.0-gl -y

curl https://sh.rustup.rs -sSf | bash -s -- -y
$HOME/.cargo/bin/cargo run -p setup-orangepi