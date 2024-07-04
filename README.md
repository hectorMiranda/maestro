
# Maestro

## About

Maestro is a virtual tutor designed to help you learn music for the Minux platform. This repository includes several programs, including a curses-based text editor and MIDI utilities.

## Features

- Curses-based text editor
- MIDI utilities
- Configuration management

## Requirements

- Python 3.6 or higher
- `curses` library (pre-installed with Python on Unix-based systems)

## Installation

1. **Clone the Repository**:
   ```sh
   git clone https://github.com/hectorMiranda/maestro.git
   cd maestro
   ```

2. **Install Dependencies**:
   There are no additional dependencies to install.

3. **Create Configuration File**:
   Ensure you have a `config.json` file in the root directory of the project with the following structure:
   ```json
   {
       "directories": {
           "root": "WP51_ROOT",
           "plugins": "plugins"
       },
       "colors": {
           "splash_box": [1, "COLOR_WHITE", "COLOR_BLUE"],
           "menu": [2, "COLOR_BLACK", "COLOR_CYAN"]
       },
       "menu_items": {
           "f": ["File", ["New", "Open", "Save", "Exit"]],
           "e": ["Edit", ["Undo", "Cut", "Copy", "Paste"]]
       },
       "splash_screen": [
           {"text": "Welcome to Maestro", "line": 5},
           {"text": "Press any key to continue...", "line": 7}
       ]
   }
   ```

## Running the Programs

### Curses-Based Text Editor

#### On macOS (including Mac M1)

1. **Ensure Python 3 is Installed**:
   You can install Python 3 using Homebrew:
   ```sh
   brew install python
   ```

2. **Run the Program**:
   ```sh
   python3 wp.py
   ```

#### On Linux

1. **Ensure Python 3 is Installed**:
   You can install Python 3 using your package manager. For example, on Debian-based systems:
   ```sh
   sudo apt update
   sudo apt install python3
   ```

2. **Run the Program**:
   ```sh
   python3 wp.py
   ```

#### On Windows

1. **Install Python 3**:
   Download and install Python 3 from the official website: [python.org](https://www.python.org/downloads/windows/).

2. **Install `windows-curses` Package**:
   Windows does not include the `curses` library by default. You need to install it:
   ```sh
   pip install windows-curses
   ```

3. **Run the Program**:
   ```sh
   python wp.py
   ```

### MIDI Utilities

To use the MIDI utilities provided in the `midi` directory, run the corresponding Python scripts. For example:

```sh
python midi/midi_script.py
```

## Notes

- Ensure the terminal window is large enough to display the text editor interface correctly.
- For optimal performance, use a terminal emulator that supports `curses`.
- The program assumes the existence of a `config.json` file in the root directory with the correct structure.

## Troubleshooting

- If the program does not start or throws an error, ensure all dependencies are correctly installed and the `config.json` file is correctly formatted.
- On macOS, if you encounter issues related to permissions or execution, try using `sudo` to run the program:
  ```sh
  sudo python3 wp.py
  ```

## License

This project is licensed under the MIT License.

## Contributing

Feel free to fork this repository and submit pull requests. Contributions are welcome!

## Author

- Hector Miranda
- GitHub: [hectorMiranda](https://github.com/hectorMiranda)
- Email: your.email@example.com
