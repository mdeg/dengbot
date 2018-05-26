FROM rust:1.26.0
WORKDIR /main
COPY . .

RUN cargo install
CMD ["dengbot"]
