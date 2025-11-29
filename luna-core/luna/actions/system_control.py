import subprocess

def _get_current_volume() -> int:
    try:
        result = subprocess.run(
            ["osascript", "-e", "output volume of (get volume settings)"],
            capture_output=True,
            text=True,
            check=True
        )
        return int(result.stdout.strip())
    except:
        return 50 # Default fallback

def _set_volume(level: int):
    subprocess.run(
        ["osascript", "-e", f"set volume output volume {level}"],
        check=True
    )

def execute(params: dict) -> str:
    """
    Controls system settings like volume or brightness.
    
    Args:
        params (dict): 'type', 'direction', 'amount'.
        
    Returns:
        str: Status message.
    """
    control_type = params.get("type")
    direction = params.get("direction")
    amount = params.get("amount", 10)
    
    if control_type == "volume":
        try:
            current = _get_current_volume()
            if direction == "up":
                new_vol = min(100, current + amount)
            elif direction == "down":
                new_vol = max(0, current - amount)
            else:
                return "Error: Invalid direction for volume"
                
            _set_volume(new_vol)
            return f"Volume set to {new_vol}"
        except Exception as e:
            return f"Error setting volume: {str(e)}"

    elif control_type == "brightness":
        # Check if 'brightness' tool is available
        try:
            # This is a placeholder as 'brightness' CLI might not be installed.
            # We attempt to run it.
            subprocess.run(["brightness", "-l"], capture_output=True, check=True)
            
            # Logic for brightness would go here if we could parse current level easily.
            # For now, we return the requested fallback.
            return "Brightness control requires 'brightness' CLI tool (not implemented fully)"
            
        except FileNotFoundError:
            return "Brightness not supported (tool not found)"
        except Exception as e:
            return f"Error controlling brightness: {str(e)}"

    return f"Error: Unknown control type '{control_type}'"
