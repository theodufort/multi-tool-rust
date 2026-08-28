# ---- Build stage ----
FROM rust:1.86 AS build
WORKDIR /app

# Copy manifests first to leverage layer caching
COPY Cargo.toml ./
COPY src ./src

# Download and compile dependencies (cached unless Cargo.toml changes)
RUN cargo build --release --locked 2>/dev/null || cargo build --release

# Copy everything and build the final binary (keeps target/ artifacts)
COPY templates ./templates
COPY static ./static
COPY Rocket.toml ./Rocket.toml
RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim
WORKDIR /app

# Non-root user
RUN useradd --create-home appuser

COPY --from=build /app/target/release/multi-tool-rust ./server
COPY --from=build /app/templates ./templates
COPY --from=build /app/static ./static
COPY --from=build /app/Rocket.toml ./Rocket.toml

USER appuser
EXPOSE 8091

CMD ["./server"]
