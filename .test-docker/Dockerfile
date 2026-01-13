FROM ubuntu:24.04
RUN apt-get update && apt-get install -y curl
RUN groupadd -g 1000 node 2>/dev/null || groupadd node 2>/dev/null || true && \
    useradd --shell /bin/bash --create-home -u 1000 -g 1000 node 2>/dev/null || \
    useradd --shell /bin/bash --create-home -g 1000 node 2>/dev/null || \
    useradd --shell /bin/bash --create-home node || true && \
    id node
USER node
ENV HOME=/home/node
WORKDIR /home/node
RUN rm -rf $HOME/.cargo && mkdir -p $HOME/.cargo $HOME/.cache \
  && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable \
  && . $HOME/.cargo/env \
  && rustup component add rustfmt clippy rust-analyzer