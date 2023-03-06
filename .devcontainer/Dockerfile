FROM mcr.microsoft.com/vscode/devcontainers/rust:1

ARG INSTALL_RUST="false"

RUN if [ "$INSTALL_RUST" = "true" ]; then \
      rustup default nightly && \
      rustup component add rustfmt && \
      rustup component add clippy; \
    fi