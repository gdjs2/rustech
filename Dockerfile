FROM rust:1.53.0

WORKDIR /rustech
COPY . .
RUN mkdir ~/.cargo & mv /rustech/config ~/.cargo/
RUN cargo install --path .

CMD ["rustech", "&"]