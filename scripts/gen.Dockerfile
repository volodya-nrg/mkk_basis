FROM rust:1.97-alpine
LABEL maintainer="vnrg <volodya-nrg@mail.ru>"

RUN apk update
RUN apk add bash make protoc
#RUN cargo install protobuf-codegen

WORKDIR /app
