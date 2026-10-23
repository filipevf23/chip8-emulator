# Chip-8 Emulator in Rust

A simple Chip-8 emulator written in the Rust programming language.

## What is Chip-8?

CHIP-8 is an interpreted programming language developed by Joseph Weisbecker in the mid-1970s. It was initially used on 8-bit microcomputers such as the COSMAC VIP and Telmac 1800. Programs written for CHIP-8 run on a virtual machine, making it a popular target for emulation beginners. The system features a simple 35-opcode instruction set, 4096 bytes of memory, 16 general-purpose registers, a 64x32 pixel monochrome display, and a 16-key hexadecimal keypad.

## Features

* Standard CHIP-8 instruction set implementation.
* Display rendering loop.
* Keyboard input handling mapping standard PC keys to the CHIP-8 hex keypad.

## Prerequisites

You will need the Rust toolchain (Rust and Cargo) installed on your system to build and run this emulator.

## How to Build and Run

Clone the repository and navigate into the project directory. You can build and run the emulator using Cargo:

```bash
cargo build
cargo run -- path/to/game.ch8