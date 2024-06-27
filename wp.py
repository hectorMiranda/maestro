import curses
import os
import time

def setup_directory():
    if not os.path.exists('WP51_ROOT'):
        os.makedirs('WP51_ROOT')

def draw_menu(stdscr):
    menu_items = [
        ("File", 'f'), ("Edit", 'e'), ("Search", 's'), ("Layout", 'l'), 
        ("Mark", 'm'), ("Tools", 't'), ("Font", 'o'), ("Graphics", 'g'), ("Help", 'h')
    ]
    h, w = stdscr.getmaxyx()
    x_pos = 0
    for title, hotkey in menu_items:
        # Find the position of the hotkey in the title to underline it
        hotkey_idx = title.lower().find(hotkey)
        stdscr.addstr(0, x_pos, title[:hotkey_idx], curses.A_REVERSE)
        stdscr.addstr(0, x_pos + hotkey_idx, title[hotkey_idx], curses.A_REVERSE | curses.A_UNDERLINE)
        stdscr.addstr(0, x_pos + hotkey_idx + 1, title[hotkey_idx + 1:], curses.A_REVERSE)
        x_pos += len(title) + 2  # Add spacing between menu items

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
    
def show_menu_options(stdscr, title):
    menu_options = {
        'f': ["New", "Open", "Save", "Exit"],
        'e': ["Undo", "Cut", "Copy", "Paste"],
        's': ["Find", "Replace"],
        'l': ["Margins", "Orientation"],
        'm': ["Bookmarks", "Annotations"],
        't': ["Customize", "Options"],
        'o': ["Font Size", "Font Color"],
        'g': ["Insert Image", "Resize"],
        'h': ["Help Topics", "About"]
    }
    options = menu_options.get(title.lower(), [])
    h, w = stdscr.getmaxyx()
    if options:
        menu_win = curses.newwin(len(options) + 2, 20, 1, 0)  # Position right below the menu
        menu_win.box()
        for idx, option in enumerate(options):
            menu_win.addstr(idx + 1, 1, option)
        menu_win.refresh()
        menu_win.getch()
        menu_win.clear()
        menu_win.refresh()

def draw_status_bar(stdscr, filename, pos_info):
    h, w = stdscr.getmaxyx()
    if filename is None:
        filename = "unknown"
    status = f"{filename}    {pos_info}"
    stdscr.addstr(h - 1, 0, status)
    stdscr.addstr(h - 1, w - len(status) - 1, status)
    stdscr.clrtoeol()

def add_centered_str(box, line_number, text, box_width, color_pair):
    # Calculate the centered position and add the string
    center_pos = (box_width - len(text)) // 2
    box.addstr(line_number, center_pos, text, color_pair)


def splash_screen(stdscr):
    stdscr.clear()
    curses.start_color()
    curses.init_pair(1, curses.COLOR_WHITE, curses.COLOR_CYAN)  # Main screen color for splash
    curses.init_pair(2, curses.COLOR_BLUE, curses.COLOR_WHITE)  # Box color for splash
    curses.init_pair(3, curses.COLOR_WHITE, curses.COLOR_BLUE)  # Default screen color

    # Calculate center position for the text box
    height, width = stdscr.getmaxyx()
    box_width = 50
    box_height = 12
    box_start_y = (height - box_height) // 2
    box_start_x = (width - box_width) // 2

    # Draw box and background
    stdscr.bkgd(curses.color_pair(1))
    box = stdscr.subwin(box_height, box_width, box_start_y, box_start_x)
    box.bkgd(curses.color_pair(2))
    box.box()

    texts = [
        ("WordPerfectLike", 1),
        ("0.5.1.002", 2),
        ("Marcetux", 4),
        ("GNU General Public License v2.0", 5),
        ("Marcetux", 6),
        ("Lincoln Heights, CA USA", 7),
        ("NOTE: The WPLike System is using \\WP51_ROOT", 9),
        ("Please wait *", 10)
    ]
    color_pair = curses.color_pair(2)  
    for text, line_number in texts:
        add_centered_str(box, line_number, text, box_width, color_pair)

    # Refresh box and screen to show changes
    box.refresh()
    time.sleep(3)

    # Reset the background color to default and clear the screen after the splash
    stdscr.bkgd(curses.color_pair(3))
    stdscr.clear()
    stdscr.refresh()
    
def handle_file_saving(stdscr, filename, text):
    if not filename:
        filename = "untitled.txt"
    full_path = os.path.join('WP51_ROOT', filename)
    with open(full_path, 'w') as file:
        file.write("\n".join(text))
    return filename

def handle_file_loading(stdscr, filename, text):
    full_path = os.path.join('WP51_ROOT', filename)
    if os.path.exists(full_path):
        with open(full_path, 'r') as file:
            lines = file.readlines()
        text.extend(line.strip() for line in lines)
    return text

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
    curses.curs_set(0)
    curses.noecho()
    curses.cbreak()
    stdscr.keypad(True)

    splash_screen(stdscr)
    draw_menu(stdscr)
    stdscr.refresh()

    text = []
    filename = None
    row, col = 2, 0

    while True:
        draw_status_bar(stdscr, filename if filename else "unknown", f"Doc 1 Pg 1 Ln {row} Pos {col}")
        stdscr.move(row, col + 2)
        char = stdscr.getch()

        if char == 27:  # Esc key or possibly the start of a meta sequence
            next_char = stdscr.getch()
            if next_char == -1:  # No additional character, so it was just an Esc
                if stdscr.is_wintouched():
                    stdscr.clear()
                    draw_menu(stdscr)
                    stdscr.refresh()
            else:
                menu_key = chr(next_char).lower()
                if menu_key in {'f', 'e', 's', 'l', 'm', 't', 'o', 'g', 'h'}:
                    show_menu_options(stdscr, menu_key)
        elif char == curses.KEY_F3:  # F3 to quit
            break
        elif char == curses.KEY_F1:  # F1 to save
            filename = handle_file_saving(stdscr, filename, text)
        elif char == curses.KEY_F2:  # F2 to load
            text = handle_file_loading(stdscr, filename, text)
        elif char == 10:  # Enter key
            text.append("")
            row += 1
            col = 0
        elif char == curses.KEY_BACKSPACE or char == 127:
            col = handle_backspace(text, stdscr, row, col)
        else:
            if col < curses.COLS - 1:
                stdscr.addch(row, col, char)
                if len(text) > row - 2:
                    text[row-2] += chr(char)
                else:
                    text.append(chr(char))
                col += 1

curses.wrapper(main)
