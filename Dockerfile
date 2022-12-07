FROM rust as builder
WORKDIR /usr/src/ugs-metadata-server
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/ugs-metadata-server /usr/local/bin/ugs-metadata-server
CMD ["ugs-metadata-server"]