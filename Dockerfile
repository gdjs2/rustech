FROM rust:1.53.0

WORKDIR /rustech
COPY . .
RUN cargo install --path .

CMD ["rustech", "&"]