<!-- cargo-sync-readme start -->

# dot-http

![Verify](https://github.com/bayne/dot-http/workflows/Verify/badge.svg?event=push&branch=master)
[![gitmoji](https://img.shields.io/badge/gitmoji-%20%F0%9F%98%9C%20%F0%9F%98%8D-FFDD67.svg?style=flat-square)](https://github.com/carloscuesta/gitmoji)
![Powered by Rust](https://img.shields.io/badge/Powered%20By-Rust-orange?style=flat-square)

dot-http is a text-based scriptable HTTP client. It is a simple language that resembles the actual HTTP protocol but with just a smidgen of magic to make it more practical for someone who builds and tests APIs.

![demo](https://user-images.githubusercontent.com/712014/72685883-36b2f700-3aa3-11ea-8a89-0e454391579f.gif)

## Installation

### Script

Enter the following in a command prompt:

```text,no_run
curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git bayne/dot-http
```

### Binary releases

The easiest way for most users is simply to download the prebuilt binaries.
You can find binaries for various platforms on the
[release](https://github.com/bayne/dot-http/releases) page.

### Cargo

First, install [cargo](https://rustup.rs/). Then:

```bash,no_run
$ cargo install dot-http
```

You will need to use the stable release for this to work; if in doubt run

```bash,no_run
rustup run stable cargo install dot-http
```

## Usage

See `dot-http --help` for usage.

### Vim

See this [plugin](https://github.com/bayne/vim-dot-http) to use dot-http within vim.

### The request

The request format is intended to resemble HTTP as close as possible. HTTP was initially designed to be human-readable and simple, so why not use that?

**simple.http**
```text,no_run
GET http://httpbin.org
Accept: */*
```
Executing that script just prints the response to stdout:
```text,no_run
$ dot-http simple.http
GET http://httpbin.org/get

HTTP/1.1 200 OK
access-control-allow-credentials: true
access-control-allow-origin: *
content-type: application/json
date: Sat, 18 Jan 2020 20:48:50 GMT
referrer-policy: no-referrer-when-downgrade
server: nginx
x-content-type-options: nosniff
x-frame-options: DENY
x-xss-protection: 1; mode=block
content-length: 170
connection: keep-alive

{
  "args": {},
  "headers": {
    "Accept": "*/*",
    "Host": "httpbin.org"
  },
  "url": "https://httpbin.org/get"
}
```

### Variables

Use variables to build the scripts dynamically, either pulling data from your environment file or from a previous request's response handler.

**simple_with_variables.http**
```text,no_run
POST http://httpbin.org/post
Accept: */*
X-Auth-Token: {{token}}

{
    "id": {{env_id}}
}
```

**http-client.env.json**
```text,no_run
{
    "dev": {
        "env_id": 42,
        "token": "SuperSecretToken"
    }
}
```

Note that the variables are replaced by their values
```text,no_run
$ dot-http simple_with_variables.http
POST http://httpbin.org/post

HTTP/1.1 200 OK
access-control-allow-credentials: true
access-control-allow-origin: *
content-type: application/json
date: Sat, 18 Jan 2020 20:55:24 GMT
referrer-policy: no-referrer-when-downgrade
server: nginx
x-content-type-options: nosniff
x-frame-options: DENY
x-xss-protection: 1; mode=block
content-length: 342
connection: keep-alive

{
  "args": {},
  "data": "{\r\n    \"id\": 42\r\n}",
  "files": {},
  "form": {},
  "headers": {
    "Accept": "*/*",
    "Content-Length": "18",
    "Host": "httpbin.org",
    "X-Auth-Token": "SuperSecretToken"
  },
  "json": {
    "id": 42
  },
  "url": "https://httpbin.org/post"
}
```

### Environment file

Use an environment file to control what initial values variables have

**http-client.env.json**
```text,no_run
{
    "dev": {
        "host": localhost,
        "token": "SuperSecretToken"
    },
    "prod": {
        "host": example.com,
        "token": "ProductionToken"
    }
}
```

**env_demo.http**
```text,no_run
GET http://{{host}}
X-Auth-Token: {{token}}
```

Specifying different environments when invoking the command results in different values
for the variables in the script

```text,no_run
$ dot-http -e dev env_demo.http
GET http://localhost
X-Auth-Token: SuperSecretToken

$ dot-http -e prod env_demo.htp
GET http://example.com
X-Auth-Token: ProductionToken
```

### Response handler

Use previous requests to populate some of the data in future requests

**response_handler.http**
```text,no_run
POST http://httpbin.org/post
Content-Type: application/json

{
    "token": "sometoken",
    "id": 237
}

> {%
   client.global.set('auth_token', response.body.json.token);
   client.global.set('some_id', response.body.json.id);
%}

###

PUT http://httpbin.org/put
X-Auth-Token: {{auth_token}}

{
    "id": {{some_id}}
}
```

Data from a previous request

```text,no_run
$ dot-http test.http
POST http://httpbin.org/post

HTTP/1.1 200 OK
access-control-allow-credentials: true
access-control-allow-origin: *
content-type: application/json
date: Sat, 18 Jan 2020 21:01:59 GMT
referrer-policy: no-referrer-when-downgrade
server: nginx
x-content-type-options: nosniff
x-frame-options: DENY
x-xss-protection: 1; mode=block
content-length: 404
connection: keep-alive

{
  "args": {},
  "data": "{\r\n    \"token\": \"sometoken\",\r\n    \"id\": 237\r\n}",
  "files": {},
  "form": {},
  "headers": {
    "Accept": "*/*",
    "Content-Length": "46",
    "Content-Type": "application/json",
    "Host": "httpbin.org"
  },
  "json": {
    "id": 237,
    "token": "sometoken"
  },
  "url": "https://httpbin.org/post"
}
```

Can populate data in a future request

```text,no_run
$ dot-http -l 16 test.http
PUT http://httpbin.org/put

HTTP/1.1 200 OK
access-control-allow-credentials: true
access-control-allow-origin: *
content-type: application/json
date: Sat, 18 Jan 2020 21:02:28 GMT
referrer-policy: no-referrer-when-downgrade
server: nginx
x-content-type-options: nosniff
x-frame-options: DENY
x-xss-protection: 1; mode=block
content-length: 336
connection: keep-alive

{
  "args": {},
  "data": "{\r\n    \"id\": 237\r\n}",
  "files": {},
  "form": {},
  "headers": {
    "Accept": "*/*",
    "Content-Length": "19",
    "Host": "httpbin.org",
    "X-Auth-Token": "sometoken"
  },
  "json": {
    "id": 237
  },
  "url": "https://httpbin.org/put"
}
```

## Contributing

Contributions and suggestions are very welcome!

Please create an issue before submitting a PR, PRs will only be accepted if they reference an existing issue. If you have a suggested change please create an issue first so that we can discuss it.

## License
[Apache License 2.0](https://github.com/bayne/dot-http/blob/master/LICENSE)

<!-- cargo-sync-readme end -->
