
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

This program requires the latest version of Rust.
To install minutecat-cli simplt clone the repository and run:

```sh
cargo install --path ./cli
```

## Usage


## License

This program is distributed under the terms of the MIT License.

## Contributing

All contributions are welcome.
Both pull requests and issue reports are always appreciated.
Please make sure that all existing tests pass before submitting a pull request.

## TODO
- Task list
- Configurable timer (work time (default 25 minutes), small break (default 5 minutes), long break (default 30 minutes), work until long break (default 4 minutes))
- After an interval a task is marked as complete, goes in order. There can bew fewer tasks than intervals to allow an open-ended session.
- Progress bar in terminal, allow quiet mode to allow piping of stdout.
- Once started the program will not require any input until it is aborted or finished. The program should not distract you!
