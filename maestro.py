def generate_scale(root, scale_type='major'):
    # Mapping of notes and their positions on a chromatic scale starting from C
    notes = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B']
    
    # Step patterns for major and minor scales
    steps = {
        'major': [2, 2, 1, 2, 2, 2, 1],
        'minor': [2, 1, 2, 2, 1, 2, 2]
    }
    
    # Selecting the correct pattern based on the scale type
    pattern = steps[scale_type]
    
    # Finding the starting index of the root note
    start_index = notes.index(root)
    
    # Generating the scale
    scale = [root]
    current_index = start_index
    for step in pattern:
        current_index = (current_index + step) % len(notes)
        scale.append(notes[current_index])
    
    return scale

# Example usage
root_note = input("Enter the root note (e.g., C, D#, A, etc.): ")
scale_type = input("Enter the scale type ('major' or 'minor'): ")
scale = generate_scale(root_note, scale_type)
print(f"The {scale_type} scale starting from {root_note} is:", scale)
