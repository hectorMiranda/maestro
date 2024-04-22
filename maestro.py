import tkinter as tk
from tkinter import ttk

def generate_scale(root, scale_type='major'):
    notes = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
    steps = {
        'major': [2, 2, 1, 2, 2, 2, 1],
        'minor': [2, 1, 2, 2, 1, 2, 2]
    }
    pattern = steps[scale_type]
    start_index = notes.index(root)
    scale = [root]
    current_index = start_index
    for step in pattern:
        current_index = (current_index + step) % len(notes)
        scale.append(notes[current_index])
    return scale

def display_scale():
    root_note = note_combobox.get()
    scale_type = scale_type_combobox.get()
    scale = generate_scale(root_note, scale_type)
    result_label.config(text=f"The {scale_type} scale starting from {root_note} is: {', '.join(scale)}")

app = tk.Tk()
app.title("Scale Generator")

mainframe = ttk.Frame(app, padding="30 30 30 30")
mainframe.grid(column=0, row=0, sticky=(tk.W, tk.E, tk.N, tk.S))

app.columnconfigure(0, weight=1)
app.rowconfigure(0, weight=1)

# Dropdown for selecting the root note
note_combobox = ttk.Combobox(mainframe, values=['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'])
note_combobox.grid(column=2, row=1, sticky=tk.W)
note_combobox.set('C')

# Dropdown for selecting the scale type
scale_type_combobox = ttk.Combobox(mainframe, values=['major', 'minor'])
scale_type_combobox.grid(column=2, row=2, sticky=tk.W)
scale_type_combobox.set('major')

# Button to generate the scale
generate_button = ttk.Button(mainframe, text="Generate Scale", command=display_scale)
generate_button.grid(column=2, row=3, sticky=tk.W)

# Label to display the result
result_label = ttk.Label(mainframe, text="")
result_label.grid(column=2, row=4, sticky=tk.W)

# Set a min size for the window
app.minsize(400, 200)

app.mainloop()
