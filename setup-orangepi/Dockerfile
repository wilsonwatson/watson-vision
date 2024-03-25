ARG CROSS_BASE_IMAGE
FROM $CROSS_BASE_IMAGE

RUN sed -i 's/focal/jammy/g' /etc/apt/sources.list

# requirements of bindgen, see https://rust-lang.github.io/rust-bindgen/requirements.html
RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y llvm-dev libclang-dev clang 

# cross compile opencv, see https://docs.opencv.org/4.x/d0/d76/tutorial_arm_crosscompile_with_cmake.html
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y gcc-arm-linux-gnueabihf git build-essential cmake libopencv-dev clang libclang-dev curl build-essential

ENV CMAKE_PREFIX_PATH="/opencv/platforms/linux/build/install"