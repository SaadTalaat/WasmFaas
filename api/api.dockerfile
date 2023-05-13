from faas/base:latest

COPY ./src/ /faas/src/api/src
RUN cargo install --path .

WORKDIR /
RUN rm -rf /faas/src/api/

COPY ./boilerplate /faas/boilerplate/

WORKDIR /faas/boilerplate/
COPY ./config /faas/config

CMD ["faas"]
