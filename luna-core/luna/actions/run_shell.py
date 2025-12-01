import subprocess
import luna.debug as debug

def execute(params: dict) -> str:
    """
    Runs a shell command.
    
    Args:
        params (dict): Contains 'cmd' and optional 'dir'.
        
    Returns:
        str: Combined stdout and stderr.
    """
    cmd = params.get("cmd")
    cwd = params.get("dir")
    
    debug.log("Shell Command", f"Cmd: {cmd}, Dir: {cwd}")
    
    if not cmd:
        return "Error: Command is required."

    try:
        process = subprocess.Popen(
            cmd,
            shell=True,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        stdout, stderr = process.communicate()
        
        output = []
        if stdout:
            output.append(stdout)
        if stderr:
            output.append(stderr)
            
        result = "".join(output).strip()
        debug.log("Shell Output", result)
        return f"Shell output:\n{result}" if result else "Shell output: (empty)"
        
    except Exception as e:
        return f"Error running shell command: {str(e)}"
