# Unreal Engine Command Line interface

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Simple CLI tool which simplifies Unreal Engine usage from CLI. it will be single executable tool that makes easy to use it without external dependencies like Python.

It is greatly inspired by [ue4cli](https://github.com/adamrehn/ue4cli), but I have no plans for supporting Unreal Engine older that 5. In the long run, I am planing to provide similar amount of features and some additionals like command aliases specified per project and per installation.

## Install

It can be installed using Rust Cargo:

```sh 
cargo install --locked --git https://github.com/Leinnan/uec
```