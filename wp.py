import curses
import os
import time

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

    # Add text centered within the box
    box.addstr(1, (box_width - len("WordPerfectLike")) // 2, "WordPerfectLike", curses.color_pair(2))
    box.addstr(2, (box_width - len("0.5.1.002")) // 2, "0.5.1.002", curses.color_pair(2))
    box.addstr(4, (box_width - len("Marcetux")) // 2, "Marcetux", curses.color_pair(2))
    box.addstr(5, (box_width - len("GNU General Public License v2.0")) // 2, "GNU General Public License v2.0", curses.color_pair(2))
    box.addstr(6, (box_width - len("Marcetux")) // 2, "Marcetux", curses.color_pair(2))
    box.addstr(7, (box_width - len("Lincoln Heights, CA USA")) // 2, "Lincoln Heights, CA USA", curses.color_pair(2))
    box.addstr(9, (box_width - len("NOTE: The WPLike System is using \\WPLIKE51")) // 2, "NOTE: The WPLike System is using \\WPLIKE51", curses.color_pair(2))
    box.addstr(10, (box_width - len("Please wait *")) // 2, "Please wait *", curses.color_pair(2))
    
    # Refresh box and screen to show changes
    box.refresh()
    time.sleep(3)

    # Reset the background color to default and clear the screen after the splash
    stdscr.bkgd(curses.color_pair(3))
    stdscr.clear()
    stdscr.refresh()


def get_user_input(stdscr, prompt):
    stdscr.addstr(prompt)
    stdscr.refresh()
    curses.echo()
    input = stdscr.getstr()
    curses.noecho()
    return input.decode()



def main(stdscr):
    curses.curs_set(0)  # Hide cursor
    stdscr.clear()
    splash_screen(stdscr)  # Call the splash screen function to display the splash screen
    
    stdscr.clear()
    draw_menu(stdscr)

    stdscr.refresh()

    text = []
    filename = None
    row, col = 2, 0

    while True:
        draw_status_bar(stdscr, filename, "Doc 1 Pg 1 Ln {} Pos {}".format(row, col))
        stdscr.move(row, col + 2)  # Offset for line display
        char = stdscr.getch()
        
        if chr(char).lower() in {'f', 'e', 's', 'l', 'm', 't', 'o', 'g', 'h'}:  # Menu hotkeys
            show_menu_options(stdscr, chr(char))

        if char == curses.KEY_F3:  # F3 to quit
            break
        elif char == curses.KEY_F1:  # F1 to save
            if not filename:
                filename = get_user_input(stdscr, "\nEnter filename: ")
                if not filename:
                    stdscr.addstr("\nNo filename given, not saved!")
                    row += 2
                    continue
            with open(filename, 'w') as f:
                f.write('\n'.join(text))
            stdscr.addstr("\nFile saved as '{}'".format(filename))
            row += 2
        elif char == curses.KEY_F2:  # F2 to load
            filename = get_user_input(stdscr, "\nEnter filename to load: ")
            if os.path.exists(filename):
                with open(filename, 'r') as f:
                    text = f.read().splitlines()
                stdscr.clear()
                for idx, line in enumerate(text):
                    stdscr.addstr(2 + idx, 0, line)
                row = len(text) + 2
            else:
                stdscr.addstr("\nFile not found!")
                row += 2
        elif char == 10:  # Enter key
            text.append("")
            row += 1
            col = 0
        elif char == curses.KEY_BACKSPACE or char is 127:
            if col > 0:
                col -= 1
                text[row-2] = text[row-2][:-1]
                stdscr.delch(row, col)
        else:
            if col < curses.COLS - 1:
                stdscr.addch(row, col, char)
                if len(text) > row - 2:
                    text[row-2] += chr(char)
                else:
                    text.append(chr(char))
                col += 1

curses.wrapper(main)

