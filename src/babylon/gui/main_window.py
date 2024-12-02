"""Main window implementation for the Babylon GUI.

This module implements the primary graphical interface for The Fall of Babylon,
following the constructivist/brutalist design principles defined in AESTHETICS.md.
The interface is divided into four main panels:
- Left: Contradiction map visualization
- Center: Main detail view
- Right: Status indicators
- Bottom: Event log and command line

The design uses a Soviet-inspired color scheme with stark contrasts and
geometric shapes to reflect the game's dialectical materialist themes.
"""

from typing import Any
import tkinter as tk
from tkinter import ttk, font
import matplotlib.pyplot as plt
from matplotlib.backends.backend_tkagg import FigureCanvasTkAgg
import networkx as nx

# Color scheme from AESTHETICS.md - Soviet constructivist inspired palette
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
    """Main GUI class for the Babylon application.
    
    Implements a constructivist-inspired interface divided into four main areas:
    - Contradiction Map (left): Network visualization of dialectical relationships
    - Detail View (center): In-depth information about selected elements
    - Status Panel (right): Economic and social indicators
    - Command Interface (bottom): Event log and command line input
    
    The interface follows the brutalist/industrial aesthetic defined in AESTHETICS.md,
    using stark contrasts, geometric shapes, and a Soviet-inspired color scheme.
    """
    
    def __init__(self, root):
        """Initialize the main window and all UI components.
        
        Args:
            root: The root Tkinter window
            
        Creates the main frame layout and initializes all visualization components
        with appropriate styling based on the constructivist design principles.
        """
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
        
        Sets up ttk styles following constructivist design principles:
        - Stark contrasts between elements
        - Industrial/brutalist appearance
        - Soviet-inspired color scheme
        - Geometric sans-serif typography
        
        The styles are applied consistently across all interface elements
        to maintain the game's revolutionary aesthetic.
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
        
        Creates a matplotlib figure in the left frame showing the network
        of dialectical contradictions. The visualization uses:
        - Dark background reflecting industrial aesthetic
        - Soviet red for important relationships
        - Geometric shapes for nodes
        - Clear hierarchical layout
        
        The map updates dynamically as contradictions evolve and transform.
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
        
        Creates the primary information display area using:
        - Constructivist-inspired typography
        - High contrast color scheme
        - Industrial/brutalist styling
        - Clear information hierarchy
        
        This panel shows detailed information about:
        - Selected contradictions
        - Entity relationships
        - Historical materialist analysis
        - System state changes
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
        
        Creates a brutalist-styled panel showing key metrics:
        - Economic indicators (GDP, unemployment)
        - Production statistics
        - Class relationships
        - System stability measures
        
        Uses:
        - Monospace fonts for data
        - Soviet-inspired header styling
        - Industrial/mechanical appearance
        - Clear data visualization
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
        
        Creates a console-style log display showing:
        - System events and notifications
        - Contradiction developments
        - Class struggle progression
        - Economic system changes
        
        Styled with:
        - Monospace font for readability
        - Industrial/terminal appearance
        - Soviet-inspired headers
        - Clear event hierarchy
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
        
        Creates a command prompt for direct system interaction:
        - Enter commands to modify system state
        - Trigger events and transformations
        - Query contradiction status
        - Analyze class relationships
        
        Uses:
        - Soviet red prompt symbol (►)
        - Monospace font for commands
        - Industrial/terminal styling
        - Clear visual feedback
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
