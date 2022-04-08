FROM arm64v8/alpine:latest AS base

COPY qemu-aarch64-static /usr/bin/qemu-aarch64-static
RUN apk update && \
    apk add --no-cache pkgconfig alsa-lib-dev build-base musl-dev rustup && \
	rustup-init -yq && \
	source $HOME/.cargo/env && \
	rustup target add aarch64-unknown-linux-musl
	
WORKDIR /usr/src
RUN	source $HOME/.cargo/env && \
	cargo new sourv
WORKDIR /usr/src/sourv
COPY . .
RUN	source $HOME/.cargo/env && \
	cargo build --target aarch64-unknown-linux-musl --release --bin sourv

#Transferring build artifacts to minimal docker
FROM arm64v8/alpine:latest
EXPOSE 7878
ENV TZ=Etc/UTC 

COPY --from=base /usr/src/sourv/target/aarch64-unknown-linux-musl/release/sourv /usr/local/bin

CMD ["/usr/local/bin/sourv"]
