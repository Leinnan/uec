# Unreal Engine Command Line interface

[![build](https://github.com/Leinnan/uec/actions/workflows/rust.yml/badge.svg)](https://github.com/Leinnan/uec/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Simple CLI tool which simplifies Unreal Engine usage from CLI. it will be single executable tool that makes easy to use it without external dependencies like Python.

It is greatly inspired by [ue4cli](https://github.com/adamrehn/ue4cli), but I have no plans for supporting Unreal Engine older that 5. In the long run, I am planing to provide similar amount of features and some additionals like command aliases specified per project and per installation.

## Usage

```sh
Unreal Engine helper tool

Usage: uec [OPTIONS] <COMMAND>

Commands:
  editor                  Runs the unreal editor without an Unreal project
  build                   Builds a Unreal project
  generate-project-files  Generate a Unreal project
  editor-project          Builds and run a Unreal editor project
  set-editor              Sets the default Unreal Engine Path
  print-config            Prints the current command configuration
  help                    Print this message or the help of the given subcommand(s)

Options:
  -e, --engine-path <ENGINE_PATH>
          Override the Unreal Engine Path from config

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Install

It can be installed using Rust Cargo:

```sh
cargo install --locked --git https://github.com/Leinnan/uec
```
