import tkinter as tk
from tkinter import ttk, font
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
import networkx as nx

# Color scheme from AESTHETICS.md
COLORS = {
    'soviet_red': '#D40000',
    'near_black': '#1A1A1A',
    'off_white': '#F5F5F5',
    'gold': '#FFD700',
    'dark_red': '#8B0000',
    'dark_gray': '#404040',
    'silver': '#C0C0C0'
}

# Style configuration
STYLE = {
    'bg': COLORS['near_black'],
    'fg': COLORS['off_white'],
    'font_header': ('Futura', 14, 'bold'),
    'font_body': ('Univers', 11),
    'font_mono': ('Roboto Mono', 10),
    'padding': 10,
    'border_width': 1
}

class BabylonGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("The Fall of Babylon")
        self.root.configure(bg=STYLE['bg'])
        
        # Configure styles
        self.configure_styles()
        
        # Create main frames with styling
        self.left_frame = tk.Frame(root, bg=STYLE['bg'], 
                                 highlightbackground=COLORS['dark_gray'],
                                 highlightthickness=STYLE['border_width'])
        self.center_frame = tk.Frame(root, bg=STYLE['bg'],
                                   highlightbackground=COLORS['dark_gray'],
                                   highlightthickness=STYLE['border_width'])
        self.right_frame = tk.Frame(root, bg=STYLE['bg'],
                                  highlightbackground=COLORS['dark_gray'],
                                  highlightthickness=STYLE['border_width'])
        self.bottom_frame = tk.Frame(root, bg=STYLE['bg'],
                                   highlightbackground=COLORS['dark_gray'],
                                   highlightthickness=STYLE['border_width'])
        
        # Layout management
        self.left_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        self.center_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
        self.right_frame.pack(side=tk.LEFT, fill=tk.BOTH)
        self.bottom_frame.pack(side=tk.BOTTOM, fill=tk.X)
        
        # Initialize components
        self.setup_contradiction_map()
        self.setup_main_view()
        self.setup_status_panel()
        self.setup_event_log()
        self.setup_command_line()

    def configure_styles(self):
        """Configure the styles for the GUI elements"""
        style = ttk.Style()
        style.configure('Babylon.TLabel',
                       background=STYLE['bg'],
                       foreground=STYLE['fg'],
                       font=STYLE['font_body'])
        style.configure('BabylonHeader.TLabel',
                       background=STYLE['bg'],
                       foreground=COLORS['soviet_red'],
                       font=STYLE['font_header'])
        
    def setup_contradiction_map(self):
        """Setup the contradiction map with constructivist styling"""
        fig = plt.figure(figsize=(6,6))
        ax = fig.add_subplot(111)
        
        # Configure plot style
        fig.patch.set_facecolor(COLORS['near_black'])
        ax.set_facecolor(COLORS['near_black'])
        
        # Style the graph
        plt.style.use('dark_background')
        ax.grid(True, color=COLORS['dark_gray'], linestyle='--', alpha=0.3)
        
        # Create canvas
        canvas = FigureCanvasTkAgg(fig, self.left_frame)
        canvas.draw()
        canvas.get_tk_widget().pack(fill=tk.BOTH, expand=True, padx=STYLE['padding'], 
                                  pady=STYLE['padding'])

    def setup_main_view(self):
        """Setup the main view with constructivist-inspired styling"""
        self.details_text = tk.Text(self.center_frame,
                                  bg=STYLE['bg'],
                                  fg=STYLE['fg'],
                                  font=STYLE['font_body'],
                                  insertbackground=COLORS['soviet_red'],
                                  selectbackground=COLORS['soviet_red'],
                                  selectforeground=COLORS['off_white'],
                                  relief='flat',
                                  padx=STYLE['padding'],
                                  pady=STYLE['padding'])
        self.details_text.pack(fill=tk.BOTH, expand=True, padx=STYLE['padding'],
                             pady=STYLE['padding'])

    def setup_status_panel(self):
        """Setup the status panel with industrial/brutalist styling"""
        header = ttk.Label(self.right_frame, text="ECONOMIC INDICATORS",
                          style='BabylonHeader.TLabel')
        header.pack(pady=(STYLE['padding'], 0))
        
        metrics = [
            ("GDP", "$1.2T"),
            ("UNEMPLOYMENT", "6.2%"),
            ("PRODUCTION", "↓2.3%")
        ]
        
        for label, value in metrics:
            frame = tk.Frame(self.right_frame, bg=STYLE['bg'])
            frame.pack(fill=tk.X, padx=STYLE['padding'], pady=(STYLE['padding'], 0))
            
            ttk.Label(frame, text=label, style='Babylon.TLabel').pack(side=tk.LEFT)
            value_label = ttk.Label(frame, text=value,
                                  style='Babylon.TLabel',
                                  font=STYLE['font_mono'])
            value_label.pack(side=tk.RIGHT)

    def setup_event_log(self):
        """Setup the event log with constructivist styling"""
        frame = tk.Frame(self.bottom_frame, bg=STYLE['bg'])
        frame.pack(fill=tk.X, padx=STYLE['padding'], pady=STYLE['padding'])
        
        header = ttk.Label(frame, text="EVENT LOG",
                          style='BabylonHeader.TLabel')
        header.pack(anchor='w')
        
        self.event_log = tk.Text(frame, height=5,
                                bg=STYLE['bg'],
                                fg=COLORS['off_white'],
                                font=STYLE['font_mono'],
                                relief='flat',
                                padx=STYLE['padding'],
                                pady=STYLE['padding'])
        self.event_log.pack(fill=tk.X)

    def setup_command_line(self):
        """Setup the command line with industrial styling"""
        frame = tk.Frame(self.bottom_frame, bg=STYLE['bg'])
        frame.pack(fill=tk.X, padx=STYLE['padding'], pady=STYLE['padding'])
        
        prompt = ttk.Label(frame, text="►", 
                          foreground=COLORS['soviet_red'],
                          font=STYLE['font_mono'],
                          background=STYLE['bg'])
        prompt.pack(side=tk.LEFT)
        
        self.cmd_entry = tk.Entry(frame,
                                bg=STYLE['bg'],
                                fg=COLORS['off_white'],
                                insertbackground=COLORS['soviet_red'],
                                font=STYLE['font_mono'],
                                relief='flat')
        self.cmd_entry.pack(fill=tk.X, padx=(STYLE['padding'], 0))
        self.cmd_entry.bind('<Return>', self.process_command)

    def process_command(self, event):
        command = self.cmd_entry.get()
        # Process command here
        self.cmd_entry.delete(0, tk.END)

def main():
    root = tk.Tk()
    app = BabylonGUI(root)
    root.mainloop()

if __name__ == "__main__":
    main()
