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

def main(stdscr):
    splash_screen(stdscr)
    stdscr.clear()
    stdscr.addstr("WordPerfect-like Editor\n")
    stdscr.addstr("F1: Save  F2: Load  F3: Quit\n")
    stdscr.refresh()

    # Initialize the text storage and position
    text = []
    row, col = 2, 0

    while True:
        char = stdscr.getch()

        if char == curses.KEY_F3:  
            break
        elif char == curses.KEY_F1:  # F1 to save
            with open('output.txt', 'w') as f:
                f.write('\n'.join(text))
            stdscr.addstr(row, 0, "File saved!      ")
            row += 1
        elif char == curses.KEY_F2:  # F2 to load
            if os.path.exists('output.txt'):
                with open('output.txt', 'r') as f:
                    text = f.read().splitlines()
                for idx, line in enumerate(text):
                    stdscr.addstr(2 + idx, 0, line)
                row = len(text) + 2
            else:
                stdscr.addstr(row, 0, "File not found!  ")
                row += 1
        elif char == 10:  # Enter key
            row += 1
            col = 0
            text.append("")
        elif char == curses.KEY_BACKSPACE or char == 127:
            if col > 0:
                col -= 1
                text[row-2] = text[row-2][:-1]
                stdscr.delch(row, col)
        else:
            stdscr.addch(row, col, char)
            if row-2 < len(text):
                text[row-2] += chr(char)
            else:
                text.append(chr(char))
            col += 1

    # Clean up and close
    stdscr.refresh()
    stdscr.getch()

curses.wrapper(main)
