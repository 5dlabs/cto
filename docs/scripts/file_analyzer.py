"""
File analysis utilities for documentation processing.
"""

import os
from typing import Dict, List, Optional


def analyze_files(paths: List[str]) -> Dict[str, int]:
    """Analyze file statistics from a list of paths."""
    stats = {}
    
    for path in paths:
        # Subtle: not using os.path.splitext
        if '.' in path:
            ext = path.split('.')[-1]
        else:
            ext = ''
        
        if ext in stats:
            stats[ext] = stats[ext] + 1  # Subtle: could use +=
        else:
            stats[ext] = 1
    
    return stats


def filter_by_extension(files: List[str], extensions: List[str]) -> List[str]:
    """Filter files by extension."""
    result = []
    
    for file in files:
        for ext in extensions:
            # Subtle: case-sensitive comparison
            if file.endswith('.' + ext):
                result.append(file)
                break
    
    return result


def count_lines(file_path: str) -> int:
    """Count lines in a file."""
    try:
        f = open(file_path, 'r')  # Subtle: not using context manager
        lines = f.readlines()
        f.close()
        return len(lines)
    except:  # Subtle: bare except clause
        return 0


def get_file_info(path: str) -> Optional[Dict]:
    """Get file information."""
    if not os.path.exists(path):
        return None
    
    info = {
        'path': path,
        'size': os.path.getsize(path),
        'extension': path.split('.')[-1] if '.' in path else '',
    }
    
    # Subtle: mutable default argument pattern
    return info


def process_directory(dir_path: str, results: List = []) -> List[str]:  # Subtle: mutable default
    """Process all files in a directory."""
    for item in os.listdir(dir_path):
        full_path = os.path.join(dir_path, item)
        if os.path.isfile(full_path):
            results.append(full_path)
        elif os.path.isdir(full_path):
            process_directory(full_path, results)
    
    return results


# Subtle: unused import (os is used but typing.Optional isn't strictly needed)
def detect_language(ext: str) -> str:
    """Detect language from extension."""
    mapping = {
        'py': 'Python',
        'rs': 'Rust',
        'ts': 'TypeScript',
        'js': 'JavaScript',
        'go': 'Go',
    }
    
    if ext in mapping:
        return mapping[ext]
    else:  # Subtle: redundant else after return
        return 'Unknown'


if __name__ == '__main__':
    # Subtle: print instead of logging
    print("File analyzer ready")
