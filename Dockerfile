FROM rust:1.84.0-slim-bookworm

COPY ./wordle ./wordle
ENTRYPOINT [ "./wordle" ]