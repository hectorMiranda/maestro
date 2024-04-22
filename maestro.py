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

def display_scale():
    root_note = note_combobox.get()
    scale_type = scale_type_combobox.get()
    scale = generate_scale(root_note, scale_type)
    result_label.configure(text=f"The {scale_type} scale starting from {root_note} is: {', '.join(scale)}")


app = ctk.CTk()  # Use CTk instead of Tk
app.title("Scale Generator")

mainframe = ctk.CTkFrame(app, corner_radius=10)  # CustomTK frame
mainframe.pack(padx=20, pady=20, fill="both", expand=True)

# Dropdown for selecting the root note
note_combobox = ctk.CTkComboBox(mainframe, values=['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'])
note_combobox.pack(pady=10, fill="x")

note_combobox.set("C")

# Dropdown for selecting the scale type
scale_type_combobox = ctk.CTkComboBox(mainframe, values=['major', 'minor'])
scale_type_combobox.pack(pady=10, fill="x")

scale_type_combobox.set('major')

# Button to generate the scale
generate_button = ctk.CTkButton(mainframe, text="Generate Scale", command=display_scale)
generate_button.pack(pady=10, fill="x")

# Label to display the result
result_label = ctk.CTkLabel(mainframe, text="", wraplength=400)
result_label.pack(pady=10, fill="x")

# Set a min size for the window
app.minsize(400, 200)

app.mainloop()
