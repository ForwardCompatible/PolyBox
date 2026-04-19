#!/usr/bin/env python3
"""DIR_CREATE - Create a directory in ./workspace/"""

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

    try:
        target.mkdir(parents=True, exist_ok=True)

        print(json.dumps({
            "success": True,
            "data": {
                "path": str(target.relative_to(workspace))
            }
        }))
    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))

if __name__ == "__main__":
    main()