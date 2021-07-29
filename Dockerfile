FROM rust:1.53.0

WORKDIR /rustech
COPY . .
RUN mkdir ~/.cargo & mv config ~/.cargo/config
RUN cargo install --path .

CMD ["rustech", "&"]