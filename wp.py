import curses
import os
import time
import json
import sys


def show_modal(stdscr, message):
    h, w = stdscr.getmaxyx()
    modal_width = max(20, len(message) + 4)
    modal_height = 3
    modal_win = curses.newwin(modal_height, modal_width, (h - modal_height) // 2, (w - modal_width) // 2)
    modal_win.box()
    modal_win.addstr(1, (modal_width - len(message)) // 2, message)
    modal_win.refresh()
    modal_win.getch()
    modal_win.clear()
    modal_win.refresh()

def load_config():
    with open('config.json', 'r') as file:
        return json.load(file)
    
config = load_config()

def setup_directory():
    root_dir = config["directories"]["root"]
    if not os.path.exists(root_dir):
        os.makedirs(root_dir)

def setup_colors():
    for key, (idx, fg, bg) in config["colors"].items():
        curses.init_pair(idx, getattr(curses, fg), getattr(curses, bg))

def draw_menu(stdscr):
    menu_items = config["menu_items"]
    h, w = stdscr.getmaxyx()
    x_pos = 0
    for key, (title, _) in menu_items.items():
        hotkey = key
        hotkey_idx = title.lower().find(hotkey)
        stdscr.addstr(0, x_pos, title[:hotkey_idx], curses.A_REVERSE)
        stdscr.addstr(0, x_pos + hotkey_idx, title[hotkey_idx], curses.A_REVERSE | curses.A_UNDERLINE)
        stdscr.addstr(0, x_pos + hotkey_idx + 1, title[hotkey_idx + 1:], curses.A_REVERSE)
        x_pos += len(title) + 2


    
def show_menu_options(stdscr, title):
    options = config["menu_items"][title.lower()][1]
    h, w = stdscr.getmaxyx()
    menu_win = curses.newwin(len(options) + 2, 20, 1, 0)
    menu_win.box()
    for idx, option in enumerate(options):
        menu_win.addstr(idx + 1, 1, option)
    menu_win.refresh()
    menu_win.getch()
    menu_win.clear()
    menu_win.refresh()

def draw_status_bar(stdscr, filename, pos_info, current_task):
    
    
    h, w = stdscr.getmaxyx()
    
    if current_task is None:
        current_task = "Ready"
    
    if filename is None:
        filename = "unknown"
    status = f"{current_task} | {filename}    {pos_info}"
    stdscr.addstr(h - 1, 0, status[:w-1])  # Fill the status bar and truncate if longer than width
    stdscr.clrtoeol()  # Clear to end of line to avoid duplication or leftover characters


def add_centered_str(box, line_number, text, box_width, color_pair):
    # Calculate the centered position and add the string
    center_pos = (box_width - len(text)) // 2
    box.addstr(line_number, center_pos, text, color_pair)


def splash_screen(stdscr):
    setup_colors()
    stdscr.clear()
    height, width = stdscr.getmaxyx()
    box_width = 50
    box_height = 12
    box_start_y = (height - box_height) // 2
    box_start_x = (width - box_width) // 2
    box = stdscr.subwin(box_height, box_width, box_start_y, box_start_x)
    box.bkgd(curses.color_pair(config["colors"]["splash_box"][0]))
    box.box()

    # Example text updated to include a musical note
    splash_texts = config["splash_screen"]  # Assuming your splash text is configured in JSON
    splash_texts.append({"text": "Loading... ♪", "line": 11})  # Adding a musical note

    for item in splash_texts:
        text = item["text"]
        line_number = item["line"]
        add_centered_str(box, line_number, text, box_width, curses.color_pair(config["colors"]["splash_box"][0]))

    box.refresh()

def load_plugins():
    plugin_dir = config["directories"]["plugins"]
    if os.path.exists(plugin_dir):
        for file in os.listdir(plugin_dir):
            if file.endswith('.py'):
                plugin_name = file[:-3]
                plugin_module = __import__(f'{plugin_dir}.{plugin_name}', fromlist=[plugin_name])
                # Assume plugins are handled internally


    
def handle_file_saving(stdscr, filename, text):
    if not filename:
        filename = "untitled.txt"
    full_path = os.path.join('WP51_ROOT', filename)
    with open(full_path, 'w') as file:
        file.write("\n".join(text))
    return filename

def handle_file_loading(stdscr, filename, text):
    if not filename:
        show_modal(stdscr, "No filename provided. Press any key to continue...")
        return text  # Return existing text or empty to avoid further errors
    full_path = os.path.join('WP51_ROOT', filename)  
    if os.path.exists(full_path):
        with open(full_path, 'r') as file:
            lines = file.readlines()
        text.extend(line.strip() for line in lines)
    else:
        show_modal(stdscr, "File does not exist. Press any key to continue...")
    return text

def open_file_from_command_line(stdscr):
    if len(sys.argv) > 1:
        file_path = sys.argv[1]
        if os.path.isfile(file_path):
            filename = os.path.basename(file_path)
            return handle_file_loading(stdscr, filename, []), filename
    return [], None

def handle_backspace(text, stdscr, row, col):
    if col > 0:
        col -= 1
        text[row-2] = text[row-2][:-1]
        stdscr.delch(row, col)
    return col

def get_user_input(stdscr, prompt):
    stdscr.addstr(prompt)
    stdscr.refresh()
    curses.echo()
    input = stdscr.getstr()
    curses.noecho()
    return input.decode()

def main(stdscr):

    setup_directory()
    curses.curs_set(2)  # Cursor visible and blinking
    curses.noecho()     # Turn off auto echoing of keypress on to screen
    curses.cbreak()     # React to keys instantly, without requiring the Enter key to be pressed
    stdscr.keypad(True) # Enable keypad mode to handle special keys like arrow keys

    if 'ncursesw' in curses.__file__:
        show_modal(stdscr, "Wide character support enabled.")
    else:
        show_modal(stdscr, "Wide character support not enabled. Functionality may be limited.")



    splash_screen(stdscr)
    draw_menu(stdscr)
    stdscr.refresh()

    filename = None

    text, filename = open_file_from_command_line(stdscr)
    
    if not text:  # Initialize text if nothing was loaded
        text = [[]]
    
    row, col = 2, 0  # Start below the menu
    
    open_file_from_command_line(stdscr)
    
    while True:
        draw_status_bar(stdscr, filename if filename else "unknown", f"Doc 1 Pg 1 Ln {row} Pos {col}", None)
        stdscr.move(row, col)
        char = stdscr.getch()

        if char == 27:  # ASCII code for ESC
            draw_status_bar(stdscr, filename if filename else "unknown", f"Doc 1 Pg 1 Ln {row} Pos {col}", "ESC")

            break
        elif char == curses.KEY_UP and row > 2:  
            row -= 1
            col = min(col, len(text[row-2]))
            draw_status_bar(stdscr, filename if filename else "unknown", f"Doc 1 Pg 1 Ln {row} Pos {col}", "UP")

        elif char == curses.KEY_DOWN and row < len(text):  # Move cursor down
            row += 1
            col = min(col, len(text[row-2]))
        elif char == curses.KEY_LEFT and col > 0:  # Move cursor left
            col -= 1
        elif char == curses.KEY_RIGHT and col < len(text[row-2]):  # Move cursor right
            col += 1
        elif char == curses.KEY_F3:  # F3 to quit
            break
        elif char == curses.KEY_F1:  # F1 to save
            filename = handle_file_saving(stdscr, filename, text)
        elif char == curses.KEY_F2:  # F2 to load
            text = handle_file_loading(stdscr, filename, text)
        elif char == 10:  # Handle Enter key
            text.insert(row, [])
            row += 1
            col = 0
        elif char == curses.KEY_BACKSPACE or char == 127:  # Handle Backspace
            if col > 0:
                text[row-2].pop(col-1)
                col -= 1
            elif row > 2:
                col = len(text[row-3])
                text[row-3].extend(text.pop(row-2))
                row -= 1
        else:  # Handle regular character input
            if row-2 < len(text):
                text[row-2].insert(col, chr(char))
                col += 1
            else:
                text.append([chr(char)])
                col = 1

        # Redraw text on the screen
        stdscr.clear()
        draw_menu(stdscr)
        for r, line in enumerate(text, 2):
            stdscr.addstr(r, 0, "".join(line))
        stdscr.refresh()

curses.wrapper(main)
