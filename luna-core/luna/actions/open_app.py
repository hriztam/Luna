import subprocess

def execute(params: dict) -> str:
    """
    Opens a macOS application.
    
    Args:
        params (dict): Contains 'name' of the app.
        
    Returns:
        str: Success or error message.
    """
    app_name = params.get("name")
    if not app_name:
        return "Error: App name is required."

    try:
        # Use 'open -a "AppName"'
        subprocess.run(["open", "-a", app_name], check=True, capture_output=True, text=True)
        return f"Opened {app_name}"
    except subprocess.CalledProcessError as e:
        return f"Failed to open {app_name}: {e.stderr.strip() if e.stderr else str(e)}"
    except Exception as e:
        return f"Error opening {app_name}: {str(e)}"
