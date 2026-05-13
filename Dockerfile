# syntax=docker/dockerfile:1.7

FROM --platform=$BUILDPLATFORM golang:1.24-bookworm AS zg-builder

ARG TARGETOS=linux
ARG TARGETARCH=amd64
ARG ZG_CLIENT_REPO=https://github.com/0gfoundation/0g-storage-client.git
ARG ZG_CLIENT_REF=main

WORKDIR /src

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates git \
    && rm -rf /var/lib/apt/lists/*

RUN git clone --depth 1 --branch "${ZG_CLIENT_REF}" "${ZG_CLIENT_REPO}" client

WORKDIR /src/client

ENV CGO_ENABLED=0
ENV GOOS=$TARGETOS
ENV GOARCH=$TARGETARCH

RUN go build -trimpath -ldflags="-s -w" -o /out/0g-storage-client .


FROM --platform=$BUILDPLATFORM rust:1.94.1-bookworm AS rust-builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY Procfile ./Procfile
COPY src ./src

RUN cargo build --release --locked --bin kult_browser_backend_rust


FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --home-dir /app appuser \
    && mkdir -p /app/src/external/0g /tmp/moments \
    && chown -R appuser:appuser /app /tmp/moments

COPY --from=rust-builder /app/target/release/kult_browser_backend_rust /app/kult_browser_backend_rust
COPY --from=zg-builder /out/0g-storage-client /app/src/external/0g/0g-storage-client

RUN chmod +x /app/kult_browser_backend_rust /app/src/external/0g/0g-storage-client

ENV HOST=0.0.0.0
ENV PORT=8080
ENV ZG_BINARY_PATH=./src/external/0g/0g-storage-client

USER appuser

EXPOSE 8080

CMD ["./kult_browser_backend_rust"]
