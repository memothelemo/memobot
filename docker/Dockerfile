ARG USER=memobot
ARG RUST_BUILD_MODE=release
ARG BUILD_DIR=/usr/build/memobot

FROM lukemathwalker/cargo-chef:0.1.62-rust-1.74.0-slim-bookworm as chef
ARG BUILD_DIR
WORKDIR ${BUILD_DIR}

# Required dependencies for compilation
RUN apt update && apt install -y pkg-config

# Prepare for recipes
FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS compile
ARG BUILD_DIR
ARG RUST_BUILD_MODE
WORKDIR ${BUILD_DIR}

COPY --from=planner ${BUILD_DIR}/recipe.json recipe.json
RUN if [ "${RUST_BUILD_MODE}" = "debug" ]; then \
        cargo chef cook --recipe-path recipe.json; \
    elif [ "${RUST_BUILD_MODE}" = "release" ]; then \
        cargo chef cook --release --recipe-path recipe.json; \
    else \
        echo "Please specify whether RUST_BUILD_MODE is in 'debug' or 'release'"; \
        exit 1;\
    fi;

# Build the application
COPY . .
RUN if [ "${RUST_BUILD_MODE}" = "debug" ]; then \
        cargo build; \
    elif [ "${RUST_BUILD_MODE}" = "release" ]; then \
        cargo build --release; \
    else \
        echo "Please specify whether RUST_BUILD_MODE is in 'debug' or 'release'"; \
        exit 1;\
    fi; \
    mkdir -p dist; \
    mv target/${RUST_BUILD_MODE}/memobot dist/;

###############################################################
FROM debian:bookworm-slim AS runner

ARG BUILD_DIR
ARG USER

# songbird relies on `ytdl` binary anyway
RUN apt update && apt install -y ca-certificates

# Setup unprivileged user
RUN adduser \
    --disabled-password \
    --home "/dev/null" \
    --no-create-home \
    --gecos "" \
    ${USER}

WORKDIR /app

COPY --chmod=0755 --from=compile ${BUILD_DIR}/dist/* ./
COPY --chmod=0755 --from=compile ${BUILD_DIR}/docker/entrypoint.sh ./

USER ${USER}

LABEL org.opencontainers.image.authors="memothelemo"
LABEL org.opencontainers.image.source="https://github.com/memothelemo/assistant-bot"
LABEL org.opencontainers.image.description=""

ENTRYPOINT [ "./entrypoint.sh" ]
