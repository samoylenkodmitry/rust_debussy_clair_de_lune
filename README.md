# Rust Debussy Clair de Lune Synthesizer

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![GitHub last commit](https://img.shields.io/github/last-commit/samoylenkodmitry/rust_debussy_clair_de_lune)

## Preview

[![Clair de Lune Preview](https://img.youtube.com/vi/l-EPGSopoZE/0.jpg)](https://www.youtube.com/watch?v=l-EPGSopoZE)

This project is a Rust-based synthesizer that plays Debussy's Clair de Lune. It uses MIDI input to generate piano-like sounds in real-time.

## Features

- MIDI file parsing and playback
- Custom synthesizer with piano-like sound
- ADSR envelope for realistic note articulation
- Basic polyphony
- String resonance simulation

## Usage

1. Ensure you have Rust installed on your system.
2. Clone this repository.
3. Run the project with `cargo run`.

Note: This project requires a MIDI file named "DEB_CLAI.MID" in the project directory to function.

## Dependencies

- rodio: For audio output
- nodi: For MIDI file parsing and playback
- rand: For minor randomness in sound generation

## License

[MIT](https://opensource.org/licenses/MIT)

## Build Status

![Build Status](https://img.shields.io/github/workflow/status/samoylenkodmitry/rust_debussy_clair_de_lune/Rust)