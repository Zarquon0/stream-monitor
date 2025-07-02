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

# Install Java and Maven for regex -> json DFA script (for testing harness)
RUN apt install -y \
    default-jdk \ 
    maven

# Install extra packages test shell commands require
RUN apt install -y \
    acpi \
    iproute2 \
    net-tools \
    iptables

# Run bash shell start up script (source cargo)
COPY docker-helpers/startup.sh /
RUN chmod +x /startup.sh
ENTRYPOINT [ "/startup.sh" ]
SHELL ["/bin/bash", "-c"]
CMD [ "/bin/bash" ]