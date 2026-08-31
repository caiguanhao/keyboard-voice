FROM rust:bookworm AS builder

WORKDIR /src
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
      pkg-config espeak-ng libasound2-dev libx11-dev libxkbcommon-dev \
      libx11-xcb-dev libxcb1-dev libxcb-render0-dev libxcb-shape0-dev \
      libxcb-xfixes0-dev libxcb-keysyms1-dev libxkbcommon-x11-dev \
      libwayland-dev libegl1-mesa-dev libgl1-mesa-dev libudev-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* build.rs ./
COPY src ./src
COPY scripts ./scripts
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
      libasound2 libx11-6 libx11-xcb1 libxcb1 libxcb-render0 libxcb-shape0 \
      libxcb-xfixes0 libxcb-keysyms1 libxkbcommon0 libxkbcommon-x11-0 \
      libwayland-client0 libwayland-egl1 libegl1 libgl1 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/keyboard-voice /usr/local/bin/keyboard-voice
ENTRYPOINT ["/usr/local/bin/keyboard-voice"]

FROM scratch AS artifact
COPY --from=builder /src/target/release/keyboard-voice /keyboard-voice
