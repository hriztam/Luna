import os
import json
import time
import traceback
from datetime import datetime

# Enable debugging via env variable
DEBUG = os.getenv("LUNA_DEBUG", "0") == "1"

LOG_FILE = os.path.expanduser("./luna_debug.log")


def _write(message: str):
    """Internal helper for writing logs."""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    with open(LOG_FILE, "a") as f:
        f.write(f"[{timestamp}] {message}\n")


def log(section: str, data):
    """Public debug log call."""
    if not DEBUG:
        return

    try:
        if isinstance(data, (dict, list)):
            pretty = json.dumps(data, indent=2)
        else:
            pretty = str(data)

        _write(f"\n=== {section} ===\n{pretty}\n")
    except Exception:
        _write(f"[ERROR logging {section}] {traceback.format_exc()}")


def log_error(section: str, error: Exception):
    """Log an exception with traceback."""
    if not DEBUG:
        return
    
    tb = "".join(traceback.format_exception(type(error), error, error.__traceback__))
    _write(f"\n=== ERROR: {section} ===\n{tb}\n")


def log_time(section: str, start_time: float):
    """Measure time durations."""
    if not DEBUG:
        return
    elapsed = round(time.time() - start_time, 4)
    _write(f"{section} took: {elapsed}s\n")
