import curses
import os
import time


def splash_screen(stdscr):
    stdscr.clear()
    curses.start_color()
    curses.init_pair(1, curses.COLOR_WHITE, curses.COLOR_BLUE)
    stdscr.bkgd(curses.color_pair(1))
    stdscr.addstr(1, 1, "WordPerfectLike")
    stdscr.addstr(2, 1, "0.5.1.002")
    stdscr.addstr(4, 1, "Marcetux")
    stdscr.addstr(5, 1, "GNU General Public License v2.0")
    stdscr.addstr(6, 1, "Marcetux")
    stdscr.addstr(7, 1, "Lincoln Heights, CA USA")
    stdscr.addstr(9, 1, "NOTE: The WPLike System is using \WPLIKE51")
    stdscr.addstr(10, 1, "Please wait *")
    stdscr.refresh()

    # nothing to load for now
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
