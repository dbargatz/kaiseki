FROM rust:1.75.0-bookworm AS devcontainer

# Create a non-root user for the container using the given ARGs, which allows
# the X11 socket on the host to be accessed without hacky workarounds such as
# "xhost +local:", or heavyweight workarounds like using xauth. Implemented
# based on this guide: https://janert.me/guides/running-gui-applications-in-a-docker-container/
# NOTE: I believe this approach requires the UID/GID to match that of the user
#       on the host, so if you encounter issues launching the UI, ensure the
#       host user UID/GID and the container user UID/GID match.
ARG GID=1000
ARG UID=1000
ARG USERNAME=user
RUN set -x \
    && addgroup --gid ${GID} ${USERNAME} \
    && adduser --uid ${UID} --gid ${GID} --disabled-password --gecos "" ${USERNAME}

# Install packages necessary for development and for the UI to launch from
# inside the Docker container.
RUN set -x \
    && apt-get update --yes  \
    && apt-get install --yes --no-install-recommends \
        # In theory, egui should only require libgtk-3-0 (not the dev package),
        # but using the non-dev package we get no UI and the "NoGlutinConfigs"
        # error, as described here: https://github.com/emilk/egui/issues/3174
        libgtk-3-dev   \
        libxcursor1    \
        libxrandr2     \
        libxi6         \
        libx11-xcb1    \
    && apt-get autoremove --yes \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Before proceeding, switch to the non-root container user; if we perform the
# next steps as root, the rustup/cargo caches end up with files owned by root,
# which causes permissions issues for cargo and rust-analyzer when run as the
# container user.
USER ${UID}:${GID}
ENV DISPLAY=:0

# Install Rust format/lint tools.
RUN set -x \
    && rustup component add rustfmt \
    && rustup component add clippy \
    && cargo install cargo-audit
