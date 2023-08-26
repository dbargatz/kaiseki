FROM rust:1.72.0-bookworm AS devcontainer

RUN set -x \
    && addgroup --gid 1000 user \
    && adduser --uid 1000 --gid 1000 --disabled-password --gecos "" user

ARG DEBIAN_FRONTEND=noninteractive
RUN set -x \
    && apt-get update --yes  \
    && apt-get install --yes --no-install-recommends \
        libgtk-3-dev   \
        libxcursor1    \
        libxrandr2     \
        libxi6         \
        libx11-xcb1    \
    && apt-get autoremove --yes \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

USER 1000:1000
ENV DISPLAY=:0

RUN set -x \
    && rustup component add rustfmt \
    && rustup component add clippy \
    && cargo install cargo-audit
