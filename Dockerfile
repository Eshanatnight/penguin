FROM rust:1.77.0 as builder

WORKDIR /penguin
COPY . .

RUN apt-get update && apt-get -y install build-essential && apt-get install -y cmake

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:latest

WORKDIR /penguin
COPY --from=builder /penguin/target/release/penguin /usr/bin/penguin

CMD ["penguin"]
