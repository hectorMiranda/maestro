import curses
import os
import time


def splash_screen(stdscr):
    stdscr.clear()
    curses.start_color()
    curses.init_pair(1, curses.COLOR_WHITE, curses.COLOR_BLUE)  # Main screen color
    curses.init_pair(2, curses.COLOR_BLACK, curses.COLOR_WHITE)  # Box color

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
    box.addstr(1, (box_width - len("WordPerfectLike")) // 2, "WordPerfectLike")
    box.addstr(2, (box_width - len("0.5.1.002")) // 2, "0.5.1.002")
    box.addstr(4, (box_width - len("Marcetux")) // 2, "Marcetux")
    box.addstr(5, (box_width - len("GNU General Public License v2.0")) // 2, "GNU General Public License v2.0")
    box.addstr(6, (box_width - len("Marcetux")) // 2, "Marcetux")
    box.addstr(7, (box_width - len("Lincoln Heights, CA USA")) // 2, "Lincoln Heights, CA USA")
    box.addstr(9, (box_width - len("NOTE: The WPLike System is using \\WPLIKE51")) // 2, "NOTE: The WPLike System is using \\WPLIKE51")
    box.addstr(10, (box_width - len("Please wait *")) // 2, "Please wait *")
    
    # Refresh box and screen to show changes
    box.refresh()
    time.sleep(3)

def get_user_input(stdscr, prompt):
    stdscr.addstr(prompt)
    stdscr.refresh()
    curses.echo()
    input = stdscr.getstr()
    curses.noecho()
    return input.decode()


def main(stdscr):
    splash_screen(stdscr)
    stdscr.clear()
    stdscr.addstr("WordPerfect-like Editor\n")
    stdscr.addstr("F1: Save  F2: Load  F3: Quit\n")
    stdscr.refresh()

    # Initialize the text storage and position
    text = []
    row, col = 2, 0
    filename = None


    while True:
        stdscr.addstr(0, 0, "WordPerfect-like Editor (F1: Save  F2: Load  F3: Quit)            ")
        stdscr.refresh()

        if row > 1:
            stdscr.move(row, col)
        char = stdscr.getch()

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
        elif char == curses.KEY_BACKSPACE or char == 127:
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
