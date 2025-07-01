FROM ubuntu:latest
WORKDIR /home/stream-monitor/

# Install Rust and dependencies crates might require
RUN apt update && apt upgrade -y
RUN apt install -y \
  build-essential \
  pkg-config \
  libssl-dev \
  curl \
  ca-certificates
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Run bash shell start up script (source cargo)
COPY docker-helpers/startup.sh /
RUN chmod +x /startup.sh
ENTRYPOINT [ "/startup.sh" ]
SHELL ["/bin/bash", "-c"]
CMD [ "/bin/bash" ]