# dot_http

dot_http is a text-based scriptable HTTP client. It is a simple language that resembles the actual HTTP protocol but with additional features to make it practical for someone who builds and tests APIs.

## Usage

## Installation

### Script

Enter the following in a command prompt:

```
curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git bayne/dot_http
```

### Binary releases

The easiest way for most users is simply to download the prebuilt binaries.
You can find binaries for various platforms on the
[release](https://github.com/bayne/dot_http/releases) page.

### Cargo

First, install [cargo](https://rustup.rs/). Then:

```bash
$ cargo install dot_http
```

You will need to use the nightly release for this to work; if in doubt run

```bash
rustup run nightly cargo install dot_http
```

## Contributing
