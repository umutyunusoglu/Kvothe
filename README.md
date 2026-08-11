# Kvothe — An Open Source Chord Recognition Tool

Kvothe is an open source chord recognition tool developed as a hobby project. During this project I have also aimed to put forward an open source framework for music theoretical computation in Rust.

## Project Structure

Kvothe was built upon three submodules.

- **Uverture**: For music theoretical computation and signal processing.
- **Felurian**: For microphone input handling and stream building.
- **Kvothe**: The application layer of the project. UI and application logic is implemented here.

### Uverture

Uverture module is the core module that other parts of Kvothe are built upon. Uverture is composed of two submodules, music and signal processor modules.

The music submodule provides an API for music theoretical calculations in addition to primitive types such as Letter Symbols, Accidentals and composite types such as Notes, Chords and Scales. The provided API includes fundamental calculations for calculating pitch classes, semitone operations and other fundamental music theoretical applications.

The signal processor submodule provides a pipeline that outputs the chromagram of a given audio signal. It also encapsulates the complex logic of fourier transforms and other calculations into a simple API.

### Felurian

Felurian is a utility module that handles microphone input stream handling. It uses cpal primitive streams to create a ringbuffer in order to provide a stream that works in a windowing manner. Also, with exposed macros, the API supports WebAssembly applications too.

### Kvothe

Kvothe handles the application flow and user interface of the application. Kvothe is designed as a TUI (Terminal User Interface) application using ratatui but it supports web builds with ratzilla as well. Kvothe processes the input stream provided by Felurian and detects the notes and chords using Uverture in real time. Lastly it visualizes the data stream and its contents in a user interface.

### Use of AI

During the development of Kvothe, one of the main principles was to use human expertise for coding as much as possible. The only usage of AI coding was for the hosting of the project with Vercel and WASM porting of the Felurian module for hosting Kvothe for web interfaces.
