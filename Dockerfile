FROM ubuntu:22.04

RUN apt update && TZ=Us/Denver DEBIAN_FRONTEND=noninteractive apt install libopencv-dev clang libclang-dev curl build-essential -y

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

COPY . /usr/src/watson_vision
WORKDIR /usr/src/watson_vision

RUN $HOME/.cargo/bin/cargo run -p setup-docker

# TODO setup v4l2 and gstreamer
# TODO custom build for opencv?
# TODO entrypoint for testing

CMD [ "/usr/bin/watson-vision" ]