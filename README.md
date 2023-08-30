![Docker Build](https://github.com/igor47/smrs/actions/workflows/publish.yml/badge.svg)

# SMRS

This repository contains the source code for a cgi-dispatched link shortener written in rust.
You can host this project in your own environment using the instructions below.
Alternatively, you can use my hosted version at [https://mmrs.link](https://mmrs.link).

## Why SMRS

Links on the web are getting ever more bloated.
Don't participate!
Shorten those bastards.

However, third-party link services are a privacy nightmare.
Whether you self-host or use my hosted version, you can be sure that your clicks/opens are not being tracked.

This project is meant to be minimal and simple.
* We allow a real web server (Apache2) to handle TLS termination and static file serving.
* We execute the rust binary via CGI to implement the API and interact with [sqlite](https://sqlite.org/index.html).
* The front-end uses Apache's [SSIs](https://httpd.apache.org/docs/current/howto/ssi.html) to render static pages with a common header.
* We use a little bit of [Alpine.js](https://alpinejs.dev/) for interactive elements.
* There is no login, but you can see and set your own session cookie to keep track of your shortened links.

## Installation

The recommended way to install this project is to use the docker image.
However, especially if you just want to use the API, you can also install the binary directly in your own web server.

### Via Docker

First, prepare a directory for the data file.
It must be owned by the user with UID 82 (www-data on Debian/Ubuntu).

```bash
$ mkdir -p ${STORAGE}/smrs/data
$ sudo chown 82:82 ${STORAGE}/smrs/data
```

Here's an example docker-compose stanza:

```yaml
  smrs:
    image: ghcr.io/igor47/smrs:v0.1.1
    restart: unless-stopped
    container_name: smrs
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.smrs.rule=Host(`example.com`,`www.example.com`)"
      - "traefik.http.routers.smrs.tls=true"
      - "traefik.http.routers.smrs.tls.certresolver=letsencrypt"
      - "traefik.http.routers.smrs.entrypoints=https"
      - "traefik.http.services.smrs.loadbalancer.server.port=8000"
    volumes:
      - ${STORAGE}/smrs/data:/smrs/data
```

This uses Traefik as a reverse proxy to the container port 8000.
You can use your own reverse proxy, or expose the port directly.

### Via Binary

Clone the repository and build the binary:

```bash
$ cargo build --release
```

Put the file `target/release/smrs` somewhere in your web server's path.
You'll need to configure your server to run the binary as a CGI script.
I use `apache2` inside the container.
The config file is in [conf/smrs.conf](https://github.com/igor47/smrs/blob/master/conf/smrs.conf) in this repo.

## API

The rust binary implements the following API endpoints:

* `GET /session` - returns the current session as `{ session: string }`
* `POST /session` - sets the session cookie to the given value `{ session: string }`
* `POST /save` - saves the URL at the given token (if not already in-use) `{ url: string, token: string }`
* `GET /to/<token` - redirects to the URL at the given token, or returns a `404`
* `GET /list` - returns a list of all tokens and URLs saved by the current `session` as `{ links: [{ token: string, url: string, created_at: i32 }]`
* `POST /forget` - marks the given token as deleted, or `404`: `{ token: string }`

## Development

To run this project locally, you can install the `cargo make` tool and run the `devenv` task:

```bash
$ cargo make devenv
```

The dev environment will be accessible at [localhost:8000/](http://localhost:8000).
This will bind-mount the `htdocs` dir into the container so you can just iterate on the code and reload.
To iterate on the rust binary, you can run the `iter` task, which will build the binary locally and copy into the container:

```bash
$ cargo make iter
```

## Pull Requests

Welcome!
But keep in mind that this project is meant to be minimal.
I will not accept PRs that add too many dependencies or features that I don't want.

## License

MIT.
Feel free to fork and do what you want.
