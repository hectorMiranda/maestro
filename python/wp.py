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

def show_fast_modal(stdscr, message):
    h, w = stdscr.getmaxyx()
    modal_width = max(20, len(message) + 4)
    modal_height = 3
    modal_win = curses.newwin(modal_height, modal_width, (h - modal_height) // 2, (w - modal_width) // 2)
    modal_win.box()
    modal_win.addstr(1, (modal_width - len(message)) // 2, message)
    modal_win.refresh()
    time.sleep(1.5)
    modal_win.clear()
    modal_win.refresh()

def get_user_input(stdscr, prompt):
    h, w = stdscr.getmaxyx()
    modal_width = max(30, len(prompt) + 20)
    modal_height = 5
    modal_win = curses.newwin(modal_height, modal_width, (h - modal_height) // 2, (w - modal_width) // 2)
    curses.echo()
    modal_win.box()
    modal_win.addstr(1, 2, prompt)
    modal_win.refresh()
    user_input = modal_win.getstr(2, 2, 20)
    curses.noecho()
    modal_win.clear()
    stdscr.refresh()
    return user_input.decode()

def show_file_selector(stdscr, files):
    h, w = stdscr.getmaxyx()
    modal_width = max(40, max(len(f) for f in files) + 4)
    modal_height = len(files) + 4
    modal_win = curses.newwin(modal_height, modal_width, (h - modal_height) // 2, (w - modal_width) // 2)
    modal_win.box()
    selected = 0
    while True:
        for idx, file in enumerate(files):
            if idx == selected:
                modal_win.addstr(idx + 2, 2, file, curses.A_REVERSE)
            else:
                modal_win.addstr(idx + 2, 2, file)
        modal_win.refresh()
        key = modal_win.getch()
        if key == curses.KEY_UP and selected > 0:
            selected -= 1
        elif key == curses.KEY_DOWN and selected < len(files) - 1:
            selected += 1
        elif key == curses.KEY_ENTER or key == 10:
            return files[selected]
        elif key == 27:  # ESC key to cancel
            return None

def load_config():
    with open('config.json', 'r') as file:
        return json.load(file)
    
config = load_config()

def setup_directory():
    root_dir = config["directories"]["root"]
    if not os.path.exists(root_dir):
        os.makedirs(root_dir)

def setup_colors():
    curses.start_color()
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
    #stdscr.refresh()
    
def show_menu_options(stdscr, title):
    options = config["menu_items"][title.lower()][1]
    h, w = stdscr.getmaxyx()
    menu_win = curses.newwin(len(options) + 2, 20, 1, 0)
    menu_win.box()
    for idx, option in enumerate(options):
        menu_win.addstr(idx + 1, 1, option)
    menu_win.refresh()
    menu_win.getch()
    # menu_win.clear()
    # menu_win.refresh()

def draw_status_bar(stdscr, filename, pos_info, current_task, highlight=False):
    h, w = stdscr.getmaxyx()
    
    if current_task is None:
        current_task = "Ready"
    
    if filename is None:
        filename = "unknown"
    status = f"{current_task} | {filename}    {pos_info}"
    
    if highlight:
        stdscr.addstr(h - 1, 0, status[:w-1], curses.A_BOLD)  # Highlighted status
    else:
        stdscr.addstr(h - 1, 0, status[:w-1])  # Normal status
    
    stdscr.clrtoeol()


def add_centered_str(box, line_number, text, box_width, color_pair):
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

    splash_texts = config["splash_screen"]

    for item in splash_texts:
        text = item["text"]
        line_number = item["line"]
        add_centered_str(box, line_number, text, box_width, curses.color_pair(config["colors"]["splash_box"][0]))

    box.refresh()
    time.sleep(1)


def load_plugins():
    plugin_dir = config["directories"]["plugins"]
    if os.path.exists(plugin_dir):
        for file in os.listdir(plugin_dir):
            if file.endswith('.py'):
                plugin_name = file[:-3]
                plugin_module = __import__(f'{plugin_dir}.{plugin_name}', fromlist=[plugin_name])
                # TODO: Add plugin to menu
    
def handle_file_saving(stdscr, filename, text):
    if not filename:
        filename = get_user_input(stdscr, "Enter filename: ")
    abspath = os.path.abspath(__file__) 
    dname = os.path.dirname(abspath)
    
    full_path = os.path.join(dname, filename)
    with open(full_path, 'w') as file:
        for line in text:
            file.write("".join(line) + "\n")  # Join each line and append a newline character
    return filename

def handle_file_loading(stdscr, filename, text):
    if not filename:
        show_modal(stdscr, "No filename provided. Press any key to continue...")
        return text  # Return existing text or empty to avoid further errors
    full_path = os.path.join('.', filename)  
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

def main(stdscr):
    stdscr = curses.initscr()
    curses.start_color()
    
    setup_directory()
    setup_colors()

    curses.curs_set(2)  # Cursor visible and blinking
    curses.noecho()     # Turn off auto echoing of keypress on to screen
    curses.cbreak()     # React to keys instantly, without requiring the Enter key to be pressed
    stdscr.keypad(True) # Enable keypad mode to handle special keys like arrow keys
    stdscr.nodelay(True)  # Set stdscr to non-blocking mode
    stdscr.timeout(100)  # Wait 100 milliseconds for a key press

    if 'ncursesw' in curses.__file__:
        show_fast_modal(stdscr, "Wide character support enabled.")
    else:
        show_fast_modal(stdscr, "Wide character support not enabled. Functionality may be limited.")

    splash_screen(stdscr)
    stdscr.clear()
    stdscr.refresh()

    filename = None

    text, filename = open_file_from_command_line(stdscr)
    
    if not text:  
        text = [[]]  # list of lists
    
    row, col = 2, 0  # Start below the menu
    
    while True:
        draw_status_bar(stdscr, filename if filename else "unknown", f"Doc 1 Pg 1 Ln {row} Pos {col}", None)
        stdscr.move(row, col)
        char = stdscr.getch()

        if char == 27:  # ASCII code for ESC
            stdscr.clear()  # Clear the screen before drawing the menu
            draw_menu(stdscr)
            stdscr.getch()  # Wait for a key press to exit the menu
            continue
        elif char == curses.KEY_UP and row > 2:
            row -= 1
            col = min(col, len(text[row - 2]))
            draw_status_bar(stdscr, filename if filename else "unknown", f"Doc 1 Pg 1 Ln {row} Pos {col}", "UP")
        elif char == curses.KEY_DOWN and row < len(text) + 1:  # Move cursor down
            row += 1
            if row <= len(text):
                col = min(col, len(text[row - 2]))
            else:
                text.append([])
                col = 0
        elif char == curses.KEY_LEFT and col > 0:  # Move cursor left
            col -= 1
        elif char == curses.KEY_RIGHT and col < len(text[row - 2]):  # Move cursor right
            col += 1
        elif char == curses.KEY_F3:  # F3 to quit
            curses.endwin()
            break
        elif char == curses.KEY_F1:  # F1 to save
            filename = handle_file_saving(stdscr, filename, text)
            continue
        elif char == curses.KEY_F2:  # F2 to load
            files = os.listdir('.')
            selected_file = show_file_selector(stdscr, files)
            if selected_file:
                text = handle_file_loading(stdscr, selected_file, [])
                filename = selected_file
        elif char == 10:  # Handle Enter key
            text.insert(row - 1, [])
            row += 1
            col = 0
        elif char == curses.KEY_BACKSPACE or char == 127:
            if col > 0:
                del text[row - 2][col - 1]
                col -= 1
            elif row > 2:
                col = len(text[row - 3])
                text[row - 3].extend(text.pop(row - 2))
                row -= 1
        elif char in [10, curses.KEY_ENTER]:  # Enter key splits the line
            text.insert(row, text[row - 2][col:])
            text[row - 2] = text[row - 2][:col]
            row += 1
            col = 0
        elif 32 <= char <= 126:  # Handle printable characters
            text[row - 2].insert(col, chr(char))
            col += 1
    
        stdscr.clear()
        h, w = stdscr.getmaxyx()
        for r, line in enumerate(text, 2):
            try:
                if r < h:  # Ensure the row is within the screen height
                    stdscr.addstr(r, 0, "".join(line[:w]))  # Ensure the line is within the screen width
            except curses.error:
                pass  # Ignore errors when adding strings outside the screen boundaries
        stdscr.refresh()

curses.wrapper(main)
