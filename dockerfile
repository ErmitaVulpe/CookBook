FROM rust:alpine as builder

WORKDIR /usr/src/cook-book

RUN apk update && apk add \
    make \
    sqlite-dev sqlite-static \
    bash curl libc-dev binaryen

COPY . .

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-leptos -y
RUN cargo install diesel_cli --no-default-features --features sqlite
RUN make

# ---
FROM alpine:latest as runner

WORKDIR /app
EXPOSE 8080/tcp

COPY --from=builder /usr/src/cook-book/build .
ENTRYPOINT ["./cook-book"]
