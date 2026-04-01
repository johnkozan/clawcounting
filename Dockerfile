FROM rust:1-bookworm AS builder

# Install Node.js and pnpm for frontend build
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && npm install -g pnpm

WORKDIR /app
COPY . .

RUN pnpm install --dir frontend
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/clawcounting /usr/local/bin/clawcounting

VOLUME /data
ENV CLAWCOUNTING_DB=/data/clawcounting.db
ENV CLAWCOUNTING_HOST=0.0.0.0

EXPOSE 3000

ENTRYPOINT ["tini", "--", "clawcounting"]
CMD ["serve"]
