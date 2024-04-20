def major_scale(root):
    # Mapping of notes and their positions on a chromatic scale starting from C
    notes = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
    
    # Whole step and half step pattern of a major scale
    steps = [2, 2, 1, 2, 2, 2, 1]
    
    # Finding the starting index of the root note
    start_index = notes.index(root)
    
    # Generating the scale
    scale = [root]
    current_index = start_index
    for step in steps:
        current_index = (current_index + step) % len(notes)
        scale.append(notes[current_index])
    
    return scale

# Example usage
root_note = input("Enter the root note (e.g., C, D#, A, etc.): ")
scale = major_scale(root_note)
print("The major scale starting from", root_note, "is:", scale)
