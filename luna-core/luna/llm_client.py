import os
import json
from groq import Groq
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

def _get_system_prompt() -> str:
    """
    Reads the system prompt from the prompts directory.
    """
    current_dir = os.path.dirname(os.path.abspath(__file__))
    prompt_path = os.path.join(current_dir, "prompts", "system_prompt.txt")
    
    try:
        with open(prompt_path, "r", encoding="utf-8") as f:
            return f.read()
    except FileNotFoundError:
        return "You are a helpful assistant. Return JSON only."

def call_llm(user_input: str) -> dict:
    """
    Calls the LLM with the user input and returns a parsed JSON dictionary.
    
    Args:
        user_input (str): The natural language command from the user.
        
    Returns:
        dict: The parsed JSON response containing the action and params,
              or an error dictionary if parsing fails.
    """
    api_key = os.getenv("GROQ_API_KEY")
    if not api_key:
        return {"action": "error", "message": "GROQ_API_KEY not found in environment variables."}

    client = Groq(api_key=api_key)
    system_prompt = _get_system_prompt()

    try:
        response = client.chat.completions.create(
            model="llama-3.1-8b-instant",
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_input}
            ],
            temperature=0.0,
            response_format={"type": "json_object"}
        )

        content = response.choices[0].message.content
        if not content:
             return {"action": "error", "message": "Empty response from LLM"}

        return json.loads(content)

    except json.JSONDecodeError:
        return {"action": "error", "message": "Invalid JSON"}
    except Exception as e:
        return {"action": "error", "message": str(e)}
