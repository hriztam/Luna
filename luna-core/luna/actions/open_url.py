import subprocess

def execute(params: dict) -> str:
    """
    Opens a URL in the default browser.
    
    Args:
        params (dict): Contains 'url'.
        
    Returns:
        str: Success or error message.
    """
    url = params.get("url")
    if not url:
        return "Error: URL is required."

    try:
        # Use 'open "url"'
        subprocess.run(["open", url], check=True, capture_output=True, text=True)
        return f"Opened {url}"
    except subprocess.CalledProcessError as e:
        return f"Failed to open {url}: {e.stderr.strip() if e.stderr else str(e)}"
    except Exception as e:
        return f"Error opening {url}: {str(e)}"
