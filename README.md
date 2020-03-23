# Super Key Loader

A small utility to copy all your GitHub SSH public keys on your system.


## How it works

Superkeyloader is a simple CLI binary written in Rust that downloads your SSH keys from GitHub and append them to your `~/.ssh/authorized_keys` file.

Use it when setting up a new machine (VPS, home server, Pi, etc) to authorize all your SSH key at the same time.


## Why?

A couple of months ago I installed Ubuntu Server 19.10. During the installation, it asked my GitHub username, and copied my SSH public keys on GitHub into my user's authorized keys.
Was great experience, I set a strong password and disabled SSH password login from the first boot. Great security and great convenience.

I searched for a tool that would do the same thing, but I found nothing. ***If you know a tool that does this thing please let me know.***

So I decided to build one my own.

The first implementation was a simple Python CLI built with [`click`](https://click.palletsprojects.com/). Cool and easy to build.
It required a Python interpreter, that it's installed on most, but not all, modern systems.
I wanted a single small binary, and packaging the Python tool wasn't practical.

Learning Rust has been on my to-do list for a long time, and this project seemed a good learning opportunity.

The result is this small Rust tool.


## Installation

1. Install from Cargo (requires Rust toolchain):

    ```
    cargo install superkeyloader
    ```

2. Download binary from [releases](https://github.com/biosan/superkeyloader/releases) page and add it to you `$PATH`. *Available for linux and macOS on "x86"*

> ***NOTE:*** Linux binaries that ends with `gnu` requires some GNU dependencies. If you want a fully self-contained binary with *no external dependencies* download the `musl` version.


## Usage

```
superkeyloader 0.1.0

USAGE:
    superkeyloader [FLAGS] [OPTIONS] <username>

FLAGS:
    -h, --help
            Prints help information

    -m, --human


    -j, --json


    -q, --quiet
            Pass many times for less log output

    -p, --stdout


    -V, --version
            Prints version information

    -v, --verbose
            Pass many times for more log output

            By default, it'll only report errors. Passing `-v` one time also prints warnings, `-vv` enables info
            logging, `-vvv` debug, and `-vvvv` trace.

OPTIONS:
    -o, --output <path>
             [default: ~/.ssh/authorized_keys]


ARGS:
    <username>
```


## Roadmap

- [ ] Build ARM binaries **IMPORTANT**
- [ ] Improve documentation and publish it
- [ ] Add a simple installation script
- [ ] Add support for external machines (like `ssh-copy-id`)
- [ ] Add support for GitLab and BitBucket
- [ ] Publish on Homebrew
- [ ] Publish on other package managers
- [ ] Add Windows support with real-world testing (if someone cares about)


## Contributing

Probably no one will ever read this, but in the rare case that you end up here and you want to add some features, improve my code, suggest a new functionality, or more probably to fill up a issue to fix a bug, etc., in any case you are welcome to make PRs, fill issues, or send me a mail.

I'm also very interested in real-world test cases and usage scenarios. Let me know if this small utility was useful to you or if you have any idea on how to improve it.


### Environment setup

Development dependencies and other small tasks are handled by [`just`](https://github.com/casey/just), a Rust-based alternative to `make`.

Steps to start contributing:

1. Install Rust toolchain on your machine. [Official guide](https://www.rust-lang.org/tools/install).
2. Install `just`
    ```
    cargo install just
    ```
3. Clone this repository (or fork and clone).
4. Setup development environment
    ```
    just setup-dev-env
    ```
    It will install:
    - [`convco`](https://github.com/hdevalke/convco) - Check that commits conform to conventional commits specification (I choose this over `commitlint` to remove non-Rust dependencies).
    - [`rusty-hook`](https://github.com/swellaby/rusty-hook) - Install git hooks
    - [`grcov`](https://github.com/mozilla/grcov) - Code coverage tool from Mozilla (I use it on macOS to get local code coverage reports, not strictly required)
    - `rust nightly` - Rust nightly toolchain, required by `grcov`
    - `clippy` - Rust code linter (`cargo` component)
    - `rustfmt` - Rust code formatter (`cargo` component)
    and setup git hooks.


### Git hooks

This repository has git hooks to enforce good formatting, code linting, and testing on developer side (thanks [`rusty-hook`](https://github.com/swellaby/rusty-hook)), the same rules will be applied on GitHub Actions.

> ***NOTE:*** `rusty-hook` should setup hooks when you first run `cargo test`.
> In case it doesn't work or after some time it stops working, you could setup hooks again with `just install-hooks`.


### Conventional commits

This repo follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/), and use [`convco`](https://github.com/hdevalke/convco) to enforce them (on developer side).
`convco` is run by the `commit-msg` hook every time you make a commit.

> ***NOTE***: It's installed by `just setup-dev-env`


#### ...on CI

All this conventions are also enforced on CI. Using [`commitlint-github-action`](https://github.com/wagoid/commitlint-github-action).

The action use [`commitlint`](https://github.com/conventional-changelog/commitlint), so there is also a very basic configuration file. If you use `commitlint` it will use it.


### Continuous Integration/Delivery

This project use GitHub Actions for CI and building releases.
Configuration is based on [`Mean Bean CI`](https://github.com/XAMPPRocky/mean-bean-ci-template) template.
Code formatting, linting and commit messages rules (see above) are enforced on CI too.

Thanks to the awesome [`action-rs`](https://github.com/action-rs) project.


### Code Coverage

Code coverage is calculated on every push by [`action-rs/grcov`](https://github.com/action-rs/grcov) and uploaded to [Coveralls](https://coveralls.io/), where you could see coverage [history](https://coveralls.io/github/biosan/superkeyloader).


## License

This project is licensed under the [MIT license](https://choosealicense.com/licenses/mit/).

