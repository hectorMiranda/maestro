import customtkinter as ctk
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

def generate_chord(root, chord_type='major'):
    scale = generate_scale(root, chord_type)
    chord = [scale[0], scale[2], scale[4]]  # basic triad chord (1st, 3rd, and 5th notes)
    return chord

def display_scale():
    root_note = note_combobox.get()
    scale_type = scale_type_combobox.get()
    scale = generate_scale(root_note, scale_type)
    result_label.configure(text=f"The {scale_type} scale starting from {root_note} is: {', '.join(scale)}")

def display_chord():
    root_note = chord_note_combobox.get()
    chord_type = chord_type_combobox.get()
    chord = generate_chord(root_note, chord_type)
    chord_result_label.configure(text=f"The {chord_type} chord from {root_note} is: {', '.join(chord)}")

app = ctk.CTk()
app.title("Music Theory Tool")

# Creating a notebook for multiple tabs
notebook = ttk.Notebook(app)
notebook.pack(fill='both', expand=True)

# Tab for Scale Generator
scale_tab = ctk.CTkFrame(notebook)
notebook.add(scale_tab, text='Scale Generator')

# Tab for Chord Generator
chord_tab = ctk.CTkFrame(notebook)
notebook.add(chord_tab, text='Chord Generator')

# Scale Generator components
note_combobox = ctk.CTkComboBox(scale_tab, values=['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'])
note_combobox.pack(pady=10, fill="x")
note_combobox.set("C")

scale_type_combobox = ctk.CTkComboBox(scale_tab, values=['major', 'minor'])
scale_type_combobox.pack(pady=10, fill="x")
scale_type_combobox.set('major')

generate_button = ctk.CTkButton(scale_tab, text="Generate Scale", command=display_scale)
generate_button.pack(pady=10, fill="x")

result_label = ctk.CTkLabel(scale_tab, text="", wraplength=400)
result_label.pack(pady=10, fill="x")

# Chord Generator components
chord_note_combobox = ctk.CTkComboBox(chord_tab, values=['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'])
chord_note_combobox.pack(pady=10, fill="x")
chord_note_combobox.set("C")

chord_type_combobox = ctk.CTkComboBox(chord_tab, values=['major', 'minor'])
chord_type_combobox.pack(pady=10, fill="x")
chord_type_combobox.set('major')

chord_generate_button = ctk.CTkButton(chord_tab, text="Generate Chord", command=display_chord)
chord_generate_button.pack(pady=10, fill="x")

chord_result_label = ctk.CTkLabel(chord_tab, text="", wraplength=400)
chord_result_label.pack(pady=10, fill="x")

app.minsize(400, 200)
app.mainloop()
