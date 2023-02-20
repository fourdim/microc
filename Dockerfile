FROM rust:alpine

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.bfsu.edu.cn/g' /etc/apk/repositories \
  && apk add sudo \
  && adduser csc4180 \
  --home /opt/csc4180-a1-119010440 \
  --disabled-password \
  && (printf 'csc4180 ALL=(ALL) NOPASSWD:ALL\n' | tee -a /etc/sudoers)

COPY --chown=csc4180:csc4180 ./ /opt/csc4180-a1-119010440/

USER csc4180

WORKDIR /opt/csc4180-a1-119010440

RUN cargo build --release

CMD ["sh"]
