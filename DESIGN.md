For brewdio I wish to build a homebrew application (recipe designer, calculator, brewing assistant, inventory manager and brewing diary tools) with the following technical constraints:

- A core written in rust. The core brewing calculations and data model will be written in rust where they can be used from many programming languages easily.
- I wish to design a somewhat retro modern terminal interface which might remind one of the data entry apps of yesteryear and seems hacker friendly. Keyboard driven (though mouse support if possible). For this we'll use the core rust library and ratatui.
- The rust core will also include a sqlite peristence layer (and eventually 2-way sync to an optional backend). I'm mostly considering sqlx here and within a webapp I guess it should use OPFS as storage. From terminal or desktop it should use a sqlite database in an XDG compliant directory.
