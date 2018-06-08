FROM rust:1.26.2
WORKDIR /main
COPY . .

RUN cargo install
CMD ["dengbot"]
