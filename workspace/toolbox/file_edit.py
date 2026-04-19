#!/usr/bin/env python3
"""FILE_EDIT - Edit a file in ./workspace/"""

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
        old_str = params.get("old_str")
        new_str = params.get("new_str")
        start_line = params.get("start_line")
        end_line = params.get("end_line")
        content = params.get("content")

        if start_line is not None and end_line is not None:
            # Line replacement mode
            with open(target, 'r') as f:
                lines = f.readlines()

            start = (start_line - 1) if start_line else 0
            end = end_line if end_line else len(lines)

            if content is not None:
                lines[start:end] = [content + '\n' if not content.endswith('\n') else content]

            target.write_text(''.join(lines))

            print(json.dumps({
                "success": True,
                "data": {"lines_modified": end - start}
            }))
        elif old_str is not None and new_str is not None:
            # String replacement mode
            file_content = target.read_text()
            if old_str not in file_content:
                print(json.dumps({"success": False, "error": "old_str not found in file"}))
                return

            new_content = file_content.replace(old_str, new_str)
            target.write_text(new_content)

            count = file_content.count(old_str)
            print(json.dumps({
                "success": True,
                "data": {"replacements": count}
            }))
        else:
            print(json.dumps({"success": False, "error": "must provide either old_str+new_str or start_line+end_line+content"}))
    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))

if __name__ == "__main__":
    main()