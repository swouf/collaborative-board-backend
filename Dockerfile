# Base image with Rust for building and running the app.
# Taken from https://github.com/rust-lang/docker-rust/blob/master/stable/bullseye/Dockerfile
FROM amazonlinux:2023.8.20250808.1 as base

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.89.0

RUN set -eux; \
    rustArch='aarch64-unknown-linux-gnu'; \
    rustupSha256='e3853c5a252fca15252d07cb23a1bdd9377a8c6f3efa01531109281ae47f841c'; \
    url="https://static.rust-lang.org/rustup/archive/1.28.2/${rustArch}/rustup-init"; \
    curl -o rustup-init "$url"; \
    # wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

FROM base as chef
WORKDIR /app
RUN yum update && yum install lld clang postgresql15-server-devel -y; \
    cargo install cargo-chef

FROM chef as planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application!
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached. 
COPY . .
# Build our project
RUN cargo build --release

# FROM base AS runtime
# WORKDIR /app
# RUN yum update -y \
#     && yum install -y openssl ca-certificates postgresql15 \
#     # Clean up
#     && yum autoremove -y
# COPY --from=builder /app/target/release/collaborative-ideation-backend collaborative-ideation-backend
# # TODO Pass the DATABASE_URL env variable
# ENV APP_ENVIRONMENT production
# ENTRYPOINT ["./collaborative-ideation-backend"]

FROM scratch AS export
COPY --from=builder /app/target/release/collaborative-ideation-backend collaborative-ideation-backend
ENTRYPOINT ["./collaborative-ideation-backend"]