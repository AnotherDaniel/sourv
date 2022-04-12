FROM arm64v8/alpine:latest AS base

# COPY qemu-aarch64-static /usr/bin/qemu-aarch64-static
WORKDIR /usr/bin
RUN apk update && \
    apk add --no-cache pkgconfig pulseaudio pulseaudio-alsa alsa-lib-dev build-base musl-dev rustup && \
	wget https://github.com/multiarch/qemu-user-static/releases/download/v6.1.0-8/qemu-aarch64-static

RUN	rustup-init -yq && \
	source $HOME/.cargo/env && \
	rustup target add aarch64-unknown-linux-musl

WORKDIR /usr/src
RUN	source $HOME/.cargo/env && \
	mkdir sourv
WORKDIR /usr/src/sourv
COPY . .
RUN	source $HOME/.cargo/env && \
	cargo build --target aarch64-unknown-linux-musl --release

# Transferring build artifacts to minimal docker
FROM arm64v8/alpine:latest
EXPOSE 7878
ENV TZ=Etc/UTC 

RUN apk update && \
    apk add --no-cache pulseaudio pulseaudio-alsa
	
ENV UNAME sourv
RUN export UNAME=$UNAME UID=1000 GID=1000 && \
	addgroup -g ${GID} ${UNAME} && \
	adduser \
	--disabled-password \
    -g "" \
    -h "/home/$UNAME" \
    -G "$UNAME" \
    -u "$UID" \
    "$UNAME" audio
COPY pulse-client.conf /etc/pulse/client.conf
USER $UNAME

COPY --from=base /usr/src/sourv/target/aarch64-unknown-linux-musl/release/main /usr/local/bin/sourv

CMD ["/usr/local/bin/sourv"]
