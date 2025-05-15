# Stage 1: Build Rust module
FROM rust:latest AS rust-builder
WORKDIR /var/


# Install Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install diesel_cli
RUN apt-get update && apt-get install default-libmysqlclient-dev libpq-dev libsqlite3-dev

COPY . .

CMD ["bash"]
