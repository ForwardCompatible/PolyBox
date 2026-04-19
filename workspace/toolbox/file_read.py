#!/usr/bin/env python3
"""FILE_READ - Read a file from ./workspace/"""

import json
import sys
from pathlib import Path

def main():
    if len(sys.argv) < 2:
        print(json.dumps({"success": False, "error": "No parameters provided"}))
        return

    try:
        params = json.loads(sys.argv[1])
    except json.JSONDecodeError:
        print(json.dumps({"success": False, "error": "Invalid JSON parameters"}))
        return

    path = params.get("path")
    if not path:
        print(json.dumps({"success": False, "error": "missing required parameter: path"}))
        return

    # Validate path is within ./workspace/
    workspace = Path(__file__).parent.parent.absolute()
    target = (workspace / path).resolve()

    if not str(target).startswith(str(workspace)):
        print(json.dumps({"success": False, "error": "path must be within ./workspace/"}))
        return

    if not target.exists():
        print(json.dumps({"success": False, "error": f"file not found: {path}"}))
        return

    try:
        start_line = params.get("start_line")
        end_line = params.get("end_line")

        if start_line is not None or end_line is not None:
            # Line range read
            with open(target, 'r') as f:
                lines = f.readlines()
            start = (start_line - 1) if start_line else 0
            end = end_line if end_line else len(lines)
            content = ''.join(lines[start:end])
            lines_returned = end - start
        else:
            # Full file read
            content = target.read_text()
            lines_returned = len(content.splitlines())

        print(json.dumps({
            "success": True,
            "data": {
                "content": content,
                "lines_returned": lines_returned
            }
        }))
    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))

if __name__ == "__main__":
    main()