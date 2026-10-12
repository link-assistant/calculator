"""Read fetched documentation without copying prose into the corpus."""
from html.parser import HTMLParser
from pathlib import Path
import sys

class Text(HTMLParser):
    def __init__(self):
        super().__init__()
        self.parts = []
        self.skip = 0
    def handle_starttag(self, tag, attrs):
        if tag in ('script', 'style'):
            self.skip += 1
        if tag in ('p', 'pre', 'tr', 'h1', 'h2', 'h3', 'li', 'div', 'br'):
            self.parts.append('\n')
        if tag in ('td', 'th'):
            self.parts.append(' | ')
    def handle_endtag(self, tag):
        if tag in ('script', 'style'):
            self.skip -= 1
        if tag in ('p', 'pre', 'tr', 'h1', 'h2', 'h3', 'li', 'div'):
            self.parts.append('\n')
    def handle_data(self, data):
        if not self.skip:
            self.parts.append(data)

for name in sys.argv[1:]:
    parser = Text()
    parser.feed(Path(name).read_text())
    Path(name + '.txt').write_text('\n'.join(line.strip() for line in ''.join(parser.parts).splitlines() if line.strip()))
