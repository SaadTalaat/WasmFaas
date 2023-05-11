FROM rust:latest

RUN groupadd -r saad -g 1000 && \
    useradd -u 1000 -r -g saad -s /sbin/nologin -c "Docker image user" saad -b /home/saad

USER root
RUN wget "https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz"
RUN tar xvf cargo-binstall-x86_64-unknown-linux-musl.tgz && rm cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN mv cargo-binstall /usr/local/cargo/bin/
RUN cargo binstall -y cargo-watch
RUN rustup target add wasm32-unknown-unknown

VOLUME ./src/api/ /home/saad/src/api
ADD ./src/api /home/saad/src/api
#RUN chown -R saad:saad /home/saad/src/api/target
USER saad
WORKDIR /home/saad/src/api
#RUN cargo build --release
CMD ["cargo", "watch", "--ignore", "./assets", "--ignore", "./boilerplate", "-x", "run --release"]
