#!/usr/bin/env python3
"""FILE_APPEND - Append content to a file in ./workspace/"""

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
    content = params.get("content")

    if not path:
        print(json.dumps({"success": False, "error": "missing required parameter: path"}))
        return

    if content is None:
        print(json.dumps({"success": False, "error": "missing required parameter: content"}))
        return

    # Validate path is within ./workspace/
    workspace = Path(__file__).parent.parent.absolute()
    target = (workspace / path).resolve()

    if not str(target).startswith(str(workspace)):
        print(json.dumps({"success": False, "error": "path must be within ./workspace/"}))
        return

    try:
        # Append content
        with open(target, 'a') as f:
            f.write(content)

        print(json.dumps({
            "success": True,
            "data": {
                "path": str(target.relative_to(workspace)),
                "bytes": len(content)
            }
        }))
    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))

if __name__ == "__main__":
    main()