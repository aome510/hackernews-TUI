FROM lukemathwalker/cargo-chef:latest-rust-1.53-alpine3.13 as planner
WORKDIR app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM lukemathwalker/cargo-chef:latest-rust-1.53-alpine3.13 as cacher
WORKDIR app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.53-alpine3.13 as builder
WORKDIR app
RUN apk --no-cache add musl-dev
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release --bin hackernews_tui

FROM scratch
WORKDIR app
COPY --from=builder /app/target/release/hackernews_tui .
CMD ["./hackernews_tui"]
