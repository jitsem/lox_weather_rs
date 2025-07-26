FROM rust:1.88.0 as builder
WORKDIR /usr/src/lox_weather_rs
COPY . .
RUN cargo install --path .
FROM debian:trixie
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/lox_weather_rs /usr/local/bin/lox_weather_rs
CMD ["lox_weather_rs"]
