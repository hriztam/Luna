from luna.actions import open_app
from luna.actions import open_url
from luna.actions import run_shell
from luna.actions import system_control
import luna.debug as debug

def execute(action_dict: dict) -> str:
    """
    Dispatches the parsed action to the appropriate handler.
    
    Args:
        action_dict (dict): The parsed action dictionary containing 'action' and 'params'.
        
    Returns:
        str: A message indicating success or failure.
    """
    action = action_dict.get("action")
    params = action_dict.get("params", {})
    
    debug.log("Executor Dispatch", f"Action: {action}, Params: {params}")
    
    if action == "error":
        return f"Error: {action_dict.get('message', 'Unknown error')}"
        
    try:
        if action == "open_app":
            return open_app.execute(params)
            
        elif action == "open_url":
            return open_url.execute(params)
            
        elif action == "run_shell":
            return run_shell.execute(params)
            
        elif action == "system_control":
            return system_control.execute(params)
            
        else:
            return f"Error: No handler for action '{action}'"
            
    except Exception as e:
        debug.log_error("Executor Error", e)
        return f"Execution failed: {str(e)}"
