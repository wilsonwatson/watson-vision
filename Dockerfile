FROM ubuntu:22.04

RUN apt install libopencv-dev clang libclang-dev -y
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

COPY . /usr/src/watson_vision
WORKDIR /usr/src/watson_vision

RUN cargo build --release

# TODO setup v4l2 and gstreamer
# TODO custom build for opencv?
# TODO entrypoint for testing