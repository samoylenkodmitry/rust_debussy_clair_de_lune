# Rust Debussy Clair de Lune Synthesizer

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

[MIT]
