import urwid
import os
import json

def load_config():
    with open('config.json', 'r') as file:
        return json.load(file)

config = load_config()

def setup_directory():
    root_dir = config["directories"]["root"]
    if not os.path.exists(root_dir):
        os.makedirs(root_dir)

class FileEditor:
    def __init__(self):
        self.text = urwid.Text("")
        self.edit = urwid.Edit()
        self.filename = None
        self.header = urwid.Text("Status: Ready")
        self.footer = urwid.Text("F1: Save | F2: Load | F3: Quit")
        self.main_frame = urwid.Frame(
            urwid.Filler(self.edit, valign='top'),
            header=self.header,
            footer=self.footer,
        )

    def main(self):
        self.loop = urwid.MainLoop(
            self.main_frame,
            unhandled_input=self.handle_input
        )
        self.loop.run()

    def handle_input(self, key):
        if key == 'f1':
            self.save_file()
        elif key == 'f2':
            self.load_file()
        elif key == 'f3':
            raise urwid.ExitMainLoop()
        self.update_status()

    def save_file(self):
        if not self.filename:
            self.filename = self.get_user_input("Enter filename: ")
        full_path = os.path.join('.', self.filename)
        with open(full_path, 'w') as file:
            file.write(self.edit.edit_text)
        self.update_status("File saved.")

    def load_file(self):
        files = os.listdir('.')
        selected_file = self.show_file_selector(files)
        if selected_file:
            self.filename = selected_file
            full_path = os.path.join('.', self.filename)
            with open(full_path, 'r') as file:
                self.edit.edit_text = file.read()
            self.update_status("File loaded.")

    def get_user_input(self, prompt):
        response = [None]

        def on_input_change(edit, text):
            response[0] = text

        def on_ok(button):
            raise urwid.ExitMainLoop()

        def on_cancel(button):
            response[0] = None
            raise urwid.ExitMainLoop()

        edit = urwid.Edit(prompt)
        ok_button = urwid.Button("OK", on_press=on_ok)
        cancel_button = urwid.Button("Cancel", on_press=on_cancel)
        pile = urwid.Pile([edit, urwid.Columns([ok_button, cancel_button])])
        filler = urwid.Filler(pile)
        loop = urwid.MainLoop(filler)
        loop.run()

        return response[0]

    def show_file_selector(self, files):
        selected = [None]

        def on_select(button, file):
            selected[0] = file
            raise urwid.ExitMainLoop()

        buttons = [urwid.Button(file, on_press=on_select, user_data=file) for file in files]
        listbox = urwid.ListBox(urwid.SimpleFocusListWalker(buttons))
        loop = urwid.MainLoop(listbox)
        loop.run()

        return selected[0]

    def update_status(self, status="Ready"):
        filename = self.filename if self.filename else "unknown"
        self.header.set_text(f"Status: {status} | File: {filename}")

if __name__ == "__main__":
    setup_directory()
    editor = FileEditor()
    editor.main()
