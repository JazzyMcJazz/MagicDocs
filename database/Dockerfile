# Builder
FROM postgres:16.2-alpine3.19 AS builder
RUN apk add git
RUN apk add build-base
RUN apk add clang15
RUN apk add llvm15-dev llvm15
WORKDIR /home
RUN git clone --branch v0.6.2 https://github.com/pgvector/pgvector.git
WORKDIR /home/pgvector
RUN make
RUN make install

# Runner
FROM postgres:16.2-alpine3.19
COPY --from=builder /usr/local/lib/postgresql/bitcode/vector.index.bc /usr/local/lib/postgresql/bitcode/vector.index.bc
COPY --from=builder /usr/local/lib/postgresql/vector.so /usr/local/lib/postgresql/vector.so
COPY --from=builder /usr/local/share/postgresql/extension /usr/local/share/postgresql/extension
COPY init-db.sh /docker-entrypoint-initdb.d/