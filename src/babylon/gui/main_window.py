from typing import Any
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

    def configure_styles(self) -> None:
        """Configure the styles for the GUI elements.
        
        Sets up ttk styles for labels and other widgets using the game's
        constructivist-inspired color scheme and typography.
        """
        style = ttk.Style()
        style.configure('Babylon.TLabel',
                       background=STYLE['bg'],
                       foreground=STYLE['fg'],
                       font=STYLE['font_body'])
        style.configure('BabylonHeader.TLabel',
                       background=STYLE['bg'],
                       foreground=COLORS['soviet_red'],
                       font=STYLE['font_header'])
        
    def setup_contradiction_map(self) -> None:
        """Setup the contradiction map visualization panel.
        
        Creates a matplotlib figure embedded in the left frame for displaying
        the network graph of contradictions. Uses constructivist styling with
        dark background and Soviet-inspired color scheme.
        """
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

    def setup_main_view(self) -> None:
        """Setup the main central view panel.
        
        Creates a text widget in the center frame for displaying detailed
        information about selected contradictions and events. Uses
        constructivist-inspired styling with custom fonts and colors.
        """
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

    def setup_status_panel(self) -> None:
        """Setup the economic indicators status panel.
        
        Creates a panel in the right frame displaying key economic metrics
        using industrial/brutalist styling. Shows GDP, unemployment rate,
        and production metrics with monospace fonts and stark colors.
        """
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

    def setup_event_log(self) -> None:
        """Setup the event log display panel.
        
        Creates a text widget in the bottom frame for displaying game events
        and notifications. Uses constructivist styling with monospace font
        and Soviet-inspired color scheme.
        """
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

    def setup_command_line(self) -> None:
        """Setup the command line input interface.
        
        Creates a command prompt in the bottom frame for entering game commands.
        Uses industrial styling with monospace font and Soviet red prompt symbol.
        Binds the Return key to process_command().
        """
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

    def process_command(self, event: Any) -> None:
        """Process a command entered in the command line.
        
        Args:
            event: The event object from the key binding
            
        Retrieves the command text from the entry widget, processes it,
        and clears the entry field. Command processing logic to be implemented.
        """
        command = self.cmd_entry.get()
        # Process command here
        self.cmd_entry.delete(0, tk.END)

def main() -> None:
    """Main entry point for the Babylon GUI application.
    
    Creates the root Tkinter window, instantiates the BabylonGUI class,
    and starts the main event loop.
    """
    root = tk.Tk()
    app = BabylonGUI(root)
    root.mainloop()

if __name__ == "__main__":
    main()
