FROM rust:alpine AS build
COPY backend /source
WORKDIR /source
RUN cargo build --release
RUN mkdir -p /install/usr/local/bin/
RUN mv target/release/ausgabenzettel /install/usr/local/bin

FROM alpine:latest
COPY --from=build /install/ /
CMD /usr/local/bin/ausgabenzettel

