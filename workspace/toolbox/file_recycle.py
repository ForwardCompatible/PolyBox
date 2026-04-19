#!/usr/bin/env python3
"""FILE_RECYCLE - Move a file to ./workspace/.recycle/"""

import json
import sys
import shutil
from pathlib import Path
import time

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
        # Ensure recycle directory exists
        recycle_dir = workspace / ".recycle"
        recycle_dir.mkdir(parents=True, exist_ok=True)

        # Create unique filename with timestamp
        timestamp = int(time.time())
        original_name = target.name
        recycled_name = f"OLD_{timestamp}_{original_name}"
        recycle_path = recycle_dir / recycled_name

        # Move file
        shutil.move(str(target), str(recycle_path))

        print(json.dumps({
            "success": True,
            "data": {
                "original_path": str(target.relative_to(workspace)),
                "recycled_path": str(recycle_path.relative_to(workspace))
            }
        }))
    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))

if __name__ == "__main__":
    main()