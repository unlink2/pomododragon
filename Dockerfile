FROM rust:buster

# setup
RUN mkdir -p /usr/src/pomododragon
WORKDIR /usr/src/pomododragon

RUN apt update -y && apt upgrade -y
RUN apt install -y git

COPY . /usr/src/pomododragon
RUN rustup default stable
RUN cargo install trunk wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown
RUN mkdir -p web/dist-libs/
RUN git clone git@github.com:jgthms/bulma.git ./web/dist-libs/bulma

EXPOSE 3000

ENV TRUNK_SERVE_ADDR=0.0.0.0
ENV TRUNK_SERVE_PORT=3080
ENV TRUNK_BUILD_RELEASE=true
ENV TRUNK_BUILD_TARGET="./web/index.html"

CMD ["trunk", "serve"]
