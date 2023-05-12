from faas/base:latest

RUN cargo install --path .

WORKDIR /
RUN rm -rf /faas/src/api/

COPY ./boilerplate /faas/boilerplate/

WORKDIR /faas/boilerplate/
COPY ./config /faas/config
RUN mkdir assets

CMD ["faas"]