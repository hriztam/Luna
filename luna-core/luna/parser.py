import luna.debug as debug

def parse_action(llm_json: dict) -> dict:
    """
    Validates and parses the JSON output from the LLM.
    
    Args:
        llm_json (dict): The raw dictionary returned by the LLM.
        
    Returns:
        dict: A cleaned dictionary with 'action' and 'params' if valid,
              or an error dictionary if invalid.
    """
    if not isinstance(llm_json, dict):
        debug.log("Parser Error", "Input is not a dictionary")
        return {"action": "error", "message": "Input must be a dictionary"}
    
    debug.log("Parser Input", llm_json)
    
    action = llm_json.get("action")
    params = llm_json.get("params")
    
    if not action:
        return {"action": "error", "message": "Missing 'action' field"}
    
    if params is None:
        params = {}
        
    if not isinstance(params, dict):
        return {"action": "error", "message": "'params' must be a dictionary"}

    # Validate specific actions
    if action == "open_app":
        if not params.get("name") or not isinstance(params["name"], str):
            return {"action": "error", "message": "open_app requires a 'name' string parameter"}
            
    elif action == "open_url":
        if not params.get("url") or not isinstance(params["url"], str):
            return {"action": "error", "message": "open_url requires a 'url' string parameter"}
            
    elif action == "run_shell":
        if not params.get("cmd") or not isinstance(params["cmd"], str):
            return {"action": "error", "message": "run_shell requires a 'cmd' string parameter"}
            
    elif action == "system_control":
        valid_types = ["volume", "brightness"]
        valid_directions = ["up", "down"]
        
        ctrl_type = params.get("type")
        direction = params.get("direction")
        amount = params.get("amount")
        
        if ctrl_type not in valid_types:
            return {"action": "error", "message": f"system_control 'type' must be one of {valid_types}"}
        if direction not in valid_directions:
            return {"action": "error", "message": f"system_control 'direction' must be one of {valid_directions}"}
        if not isinstance(amount, (int, float)):
             return {"action": "error", "message": "system_control 'amount' must be a number"}
             
    elif action == "error":
        # Pass through existing errors
        return llm_json
        
    else:
        return {"action": "error", "message": f"Unknown action: {action}"}

    result = {"action": action, "params": params}
    debug.log("Parser Result", result)
    return result
