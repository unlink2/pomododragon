
# pomododragon

![](https://github.com/unlink2/pomododragon/actions/workflows/build.yml/badge.svg)
![](https://github.com/unlink2/pomododragon/actions/workflows/test.yml/badge.svg)

## Table of content

- [Installation](#Installation)
- [Usage](#Usage)
- [License](#License)
- [Contributing](#Contributing)
- [TODO](#TODO)

## Installation

### CLI

This program requires the latest version of Rust.
To install pomododragon-cli simply clone the repository and run:

```sh
cargo install --path ./cli
```

### Web UI docker

To build the web-ui in docker run
```sh
docker build -t pomododragon .
```

## Usage

### CLI

The cli offers a simple help menu:
```sh
pomododragon --help
```

### Web UI docker

To run the web-ui in docker use the following command:
```sh
docker run -it -p 3080:3080 pomododragon
```

## License

This program is distributed under the terms of the MIT License.

## Contributing

All contributions are welcome.
Both pull requests and issue reports are always appreciated.
Please make sure that all existing tests pass before submitting a pull request.

## TODO
- Implement command patter for pomo state machine
