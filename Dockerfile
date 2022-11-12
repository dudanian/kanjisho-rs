FROM rust:latest as builder

WORKDIR /usr/src/kanjisho
COPY . .

# TODO how do I install from a workspace?
# RUN cargo install --path .
RUN cargo build -r

FROM debian:buster-slim
RUN apt-get update && apt-get install -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/kanjisho/target/release/backend /usr/local/bin/kanjisho

EXPOSE 7000
CMD ["kanjisho"]