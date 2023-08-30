# Build stage: Use the Rust image to build the binary
FROM rust:alpine3.18 as builder
RUN apk add --no-cache musl-dev
RUN mkdir -p /smrs/src
WORKDIR /smrs

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Build the binary
RUN rm -f target/release/deps/smrs*
COPY ./src ./src
RUN cargo build --release

# we set up our alpine HTTP image with directories
FROM httpd:alpine3.18 AS prep
RUN mkdir -p /smrs/conf /smrs/htdocs /smrs/data
RUN chown -R daemon:daemon /smrs/data
RUN echo "Include /smrs/conf/smrs.conf" >> /usr/local/apache2/conf/httpd.conf

# in dev, we bind-mount the conf and htdocs dirs, so we're done
FROM prep AS dev
WORKDIR /smrs

# in prod, we need to copy htdocs and conf from local, and the binary from the builder
FROM prep
COPY ./conf/smrs.conf /smrs/conf/smrs.conf
COPY ./htdocs/ /smrs/htdocs/
COPY --from=builder /smrs/target/release/smrs /smrs/htdocs/smrs.cgi
