# Real-Time Audio to MIDI Converter

This Python script uses `aubio`, `numpy`, `pyaudio`, and `mido` to convert real-time audio input into MIDI notes. It captures audio from the default input device, analyzes the pitch, and sends corresponding MIDI note messages to a specified output port.

## Requirements

- Python 3.x
- `aubio`
- `numpy`
- `pyaudio`
- `mido`

You can install the required libraries using pip:

```bash
pip install aubio numpy pyaudio mido
```

## Usage

1. Update the `MIDI_OUTPUT_PORT` constant in the script with the name of your MIDI output port. You can list available ports using `mido.get_output_names()`.

2. Run the script:

   ```bash
   python audio_to_midi.py
   ```

3. Play a musical instrument or sing into your microphone. The script will print detected MIDI notes and send them to the specified MIDI output port.

4. To stop the script, press `Ctrl+C` in the terminal.

## Configuration

- `BUFFER_SIZE`: Size of the audio buffer (default: 1024). Increase for lower CPU usage but higher latency.
- `SAMPLE_RATE`: Sampling rate of the audio input (default: 44100 Hz).
- `MIDI_OUTPUT_PORT`: Name of the MIDI output port to send notes to.

## Dependencies

- `aubio`: For pitch detection.
- `numpy`: For numerical operations on audio data.
- `pyaudio`: For capturing audio input.
- `mido`: For sending MIDI messages.

## Notes

- Ensure that your audio input device is correctly configured and working.
- The script currently sends a MIDI `note_on` message for each detected pitch. You may want to modify it to include `note_off` messages or handle sustained notes differently.

## License

This script is provided "as is", without warranty of any kind. Feel free to use and modify it as needed.
