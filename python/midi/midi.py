import aubio
import numpy as np
import pyaudio
import mido

# Constants
BUFFER_SIZE = 1024
SAMPLE_RATE = 44100
MIDI_OUTPUT_PORT = 'Your MIDI output port name here'

# Initialize PyAudio
p = pyaudio.PyAudio()

# Open stream
stream = p.open(format=pyaudio.paFloat32,
                channels=1,
                rate=SAMPLE_RATE,
                input=True,
                frames_per_buffer=BUFFER_SIZE)

# Aubio pitch detection
pitch_o = aubio.pitch("default", BUFFER_SIZE, BUFFER_SIZE // 2, SAMPLE_RATE)
pitch_o.set_unit("midi")
pitch_o.set_tolerance(0.8)

# MIDI output
midi_out = mido.open_output(MIDI_OUTPUT_PORT)

try:
    while True:
        data = stream.read(BUFFER_SIZE)
        samples = np.frombuffer(data, dtype=aubio.float_type)
        pitch = pitch_o(samples)[0]
        midi_note = int(round(pitch))
        if midi_note > 0:
            print(f"Detected MIDI note: {midi_note}")
            midi_out.send(mido.Message('note_on', note=midi_note))
except KeyboardInterrupt:
    print("Stopping")

# Close stream and PyAudio
stream.stop_stream()
stream.close()
p.terminate()
