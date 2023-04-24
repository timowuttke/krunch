FROM rust:1.69.0-slim as build
ARG BUILD_COMMAND
ARG BUILD_TYPE
WORKDIR /naporware

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get -y install build-essential libclang-dev libssl-dev pkg-config
RUN mkdir src/
RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
#COPY ./Cargo.lock .
COPY ./Cargo.toml .
RUN ${BUILD_COMMAND}
RUN rm -f src/* target/${BUILD_TYPE}/deps/backend*

COPY ./src ./src
RUN touch src/main.rs && ${BUILD_COMMAND}


FROM debian:11.3-slim
ARG BUILD_TYPE
COPY --from=build /naporware/target/${BUILD_TYPE}/backend .


CMD ["./backend"]