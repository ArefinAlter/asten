# The hosted pre-alpha gateway image. It packages binaries built by
# deploy/release/linux.Dockerfile (copied to deploy/pre-alpha/artifacts/):
#
#   artifacts/asten-prealpha   the gateway (one Asten API per visitor)
#   artifacts/desk/            apps/web built with `--mode hosted`
#
# Ubuntu 22.04 matches the build image's glibc.
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 asten \
    && mkdir -p /data && chown asten:asten /data
COPY artifacts/asten-prealpha /usr/local/bin/asten-prealpha
COPY artifacts/desk /desk
RUN chmod 755 /usr/local/bin/asten-prealpha && chmod -R a+rX /desk
USER asten
ENV PREALPHA_BIND=0.0.0.0:8080 \
    PREALPHA_DATA=/data \
    PREALPHA_DESK=/desk \
    PREALPHA_COOKIE_SECURE=true \
    RUST_LOG=asten_prealpha=info,asten_api=warn
EXPOSE 8080
VOLUME ["/data"]
ENTRYPOINT ["/usr/local/bin/asten-prealpha"]
