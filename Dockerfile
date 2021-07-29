FROM rust:1.53.0

WORKDIR /rustech
COPY . .
RUN mv config ~/.cargo/config
RUN cargo install --path .

CMD ["rustech", "&"]