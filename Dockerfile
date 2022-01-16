FROM rust as builder
WORKDIR app
COPY . .
RUN cargo build --release --bin hackernews_tui

FROM scratch
WORKDIR app
COPY --from=builder /app/target/release/hackernews_tui .
COPY ./examples/hn-tui.toml ./hn-tui.toml
CMD ["./hackernews_tui", "-l", ".", "-c", "./hn-tui.toml"]
