FROM rust:1.99-bookworm AS builder
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY services ./services
COPY apps/desktop/src-tauri ./apps/desktop/src-tauri
RUN cargo build --locked --release -p asten-api

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 asten
WORKDIR /app
COPY --from=builder /workspace/target/release/asten-api /usr/local/bin/asten-api
RUN mkdir /app/data && chown -R asten:asten /app
USER asten
ENV ASTEN_BIND=0.0.0.0:4317
ENV ASTEN_DATABASE_PATH=/app/data/asten.db
# Beyond loopback the API refuses ASTEN_AUTH=off; pairing is the default.
ENV ASTEN_AUTH=pairing
EXPOSE 4317
VOLUME ["/app/data"]
ENTRYPOINT ["asten-api"]
