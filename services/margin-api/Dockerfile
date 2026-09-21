FROM rust:1.96-bookworm AS builder
WORKDIR /workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY services ./services
COPY apps/desktop/src-tauri ./apps/desktop/src-tauri
RUN cargo build --locked --release -p margin-api

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 margin
WORKDIR /app
COPY --from=builder /workspace/target/release/margin-api /usr/local/bin/margin-api
RUN mkdir /app/data && chown -R margin:margin /app
USER margin
ENV MARGIN_BIND=0.0.0.0:4317
ENV MARGIN_DATABASE_PATH=/app/data/margin.db
EXPOSE 4317
VOLUME ["/app/data"]
ENTRYPOINT ["margin-api"]
