FROM lukemathwalker/cargo-chef:latest-rust-1.53.0 as planner
WORKDIR app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:latest-rust-1.53.0 as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.53.0 as builder
WORKDIR app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release --bin hackernews_tui

FROM rust:1.53.0 as runtime
WORKDIR app
COPY --from=builder /app/target/release/hackernews_tui /usr/local/bin
ENTRYPOINT ["/usr/local/bin/hackernews_tui"]
