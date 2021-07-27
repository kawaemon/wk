FROM rust:alpine as BUILD

RUN mkdir -p /src/src
RUN apk add --no-cache alpine-sdk
COPY ./Cargo.toml ./rust-toolchain /src
COPY ./src/ /src/src/
RUN cd /src && cargo build --release

FROM alpine
RUN mkdir -p /usr/local/bin
COPY --from=BUILD /src/target/release/wk /usr/local/bin/
ENTRYPOINT [ "/usr/local/bin/wk" ]
