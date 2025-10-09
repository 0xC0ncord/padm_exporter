FROM docker.io/library/rust:1.90.0-alpine@sha256:b4b54b176a74db7e5c68fdfe6029be39a02ccbcfe72b6e5a3e18e2c61b57ae26 AS builder
COPY --chmod=0755 . /build
RUN apk update && \
    apk add clang lld perl make && \
    export RUSTFLAGS="-C linker=clang -C link-arg=-fuse-ld=lld" && \
    cd /build && \
    cargo build --release
RUN mkdir -p /out/libs && \
    mkdir -p /out/libs-root && \
    ldd /build/target/release/padm_exporter && \
    ldd /build/target/release/padm_exporter | grep -v 'linux-vdso.so' | awk '{print $(NF-1) " " $1}' | sort -u -k 1,1 | awk '{print "install", "-D", $1, (($2 ~ /^\//) ? "/out/libs-root" $2 : "/out/libs/" $2)}' | xargs -I {} sh -c {} && \
    ls -Rla /out/libs && \
    ls -Rla /out/libs-root

FROM scratch
COPY --chown=0:0 --chmod=0755 --from=builder /build/target/release/padm_exporter /padm_exporter
COPY --from=builder /out/libs-root/ /
COPY --from=builder /out/libs/ /lib/
ENV LD_LIBRARY_PATH=/lib

ENV LC_ALL=C
LABEL org.opencontainers.image.authors=me@concord.sh

USER 1000:1000

ENTRYPOINT ["/padm_exporter", "--config", "config.yaml"]
