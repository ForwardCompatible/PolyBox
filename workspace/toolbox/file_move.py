#!/usr/bin/env python3
"""FILE_MOVE - Move/rename a file within ./workspace/"""

import json
import sys
import shutil
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

    src = params.get("src")
    dest = params.get("dest")

    if not src:
        print(json.dumps({"success": False, "error": "missing required parameter: src"}))
        return

    if not dest:
        print(json.dumps({"success": False, "error": "missing required parameter: dest"}))
        return

    # Validate paths are within ./workspace/
    workspace = Path(__file__).parent.parent.absolute()
    src_path = (workspace / src).resolve()
    dest_path = (workspace / dest).resolve()

    if not str(src_path).startswith(str(workspace)):
        print(json.dumps({"success": False, "error": "src path must be within ./workspace/"}))
        return

    if not str(dest_path).startswith(str(workspace)):
        print(json.dumps({"success": False, "error": "dest path must be within ./workspace/"}))
        return

    if not src_path.exists():
        print(json.dumps({"success": False, "error": f"source file not found: {src}"}))
        return

    try:
        # Create parent directories if needed
        dest_path.parent.mkdir(parents=True, exist_ok=True)

        # Move file
        shutil.move(str(src_path), str(dest_path))

        print(json.dumps({
            "success": True,
            "data": {
                "src": str(src_path.relative_to(workspace)),
                "dest": str(dest_path.relative_to(workspace))
            }
        }))
    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))

if __name__ == "__main__":
    main()