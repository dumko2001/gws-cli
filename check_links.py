import os
import re
from pathlib import Path

def check_links():
    basedir = Path("skills").resolve()
    docsdir = Path("docs").resolve()
    
    # regex to find markdown links: [text](path)
    link_regex = re.compile(r'\[.*?\]\((.*?)\)')
    
    broken_links = 0
    total_links = 0
    
    files_to_check = list(basedir.rglob("*.md")) + list(docsdir.rglob("*.md"))
    
    for filepath in files_to_check:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
            
        for match in link_regex.finditer(content):
            link_path = match.group(1).strip().split(' ', 1)[0]
            
            # Skip HTTP(S) links
            if link_path.startswith(('http://', 'https://')):
                continue
            
            # Skip empty links, mailto:, etc
            if not link_path or link_path.startswith(('mailto:', 'tel:')):
                continue

            # Skip absolute web paths like /chat/api/guides/...
            if link_path.startswith('/'):
                continue
                
            # Skip anchor-only links
            if link_path.startswith('#'):
                continue
                
            # Remove anchor from filename
            file_part = link_path.split('#')[0]
            if not file_part:
                continue

            total_links += 1
            target_path = (filepath.parent / file_part).resolve()
            if not target_path.exists():
                print(f"Broken link in {filepath}: {link_path} (resolved to {target_path})")
                broken_links += 1
                    
    print(f"Checked {total_links} local links.")
    if broken_links > 0:
        print(f"Found {broken_links} broken links!")
        exit(1)
    else:
        print("All local links are valid!")

if __name__ == "__main__":
    check_links()
