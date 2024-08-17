FROM rust:1.80-bookworm AS build

# create a new empty shell project
RUN USER=root cargo new --bin rust-axum-postgres
WORKDIR /rust-axum-postgres

# copy over your manifests
COPY ./Cargo.toml ./Cargo.toml
COPY ./migrations ./migrations

COPY ./src ./src
COPY ./config ./config

RUN cargo build --release

# our final base
FROM debian:bookworm-slim AS runtime

# copy the build artifact from the build stage
COPY --from=build /rust-axum-postgres/target/release/rust-axum-postgres .
COPY --from=build /rust-axum-postgres/config ./config
COPY --from=build /rust-axum-postgres/migrations ./migrations

EXPOSE 3000

# set the startup command to run your binary
CMD ["./rust-axum-postgres"]