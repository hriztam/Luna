import argparse
import sys
from luna.llm_client import call_llm
from luna.parser import parse_action
from luna.executor import execute
import luna.debug as debug
import time

def process_command(user_input: str):
    """
    Processes a single user command through the Luna pipeline.
    """
    print(f"Thinking...")
    
    # 1. Call LLM
    t0 = time.time()
    llm_response = call_llm(user_input)
    debug.log_time("LLM Call", t0)
    
    # 2. Parse Action
    t1 = time.time()
    action_dict = parse_action(llm_response)
    debug.log_time("Action Parsing", t1)
    
    # 3. Execute Action
    t2 = time.time()
    result = execute(action_dict)
    debug.log_time("Action Execution", t2)
    
    # 4. Output Result
    print(result)

def repl():
    """
    Runs the interactive Read-Eval-Print Loop.
    """
    print("Luna CLI (Interactive Mode)")
    print("Type 'exit' or 'quit' to stop.")
    
    while True:
        try:
            user_input = input("\n> ").strip()
            
            if not user_input:
                continue
                
            if user_input.lower() in ["exit", "quit"]:
                print("Goodbye!")
                break
                
            process_command(user_input)
            
        except KeyboardInterrupt:
            print("\nGoodbye!")
            break
        except Exception as e:
            debug.log_error("CLI Loop Error", e)
            print(f"An error occurred: {e}")

def main():
    parser = argparse.ArgumentParser(description="Luna - macOS Automation Assistant")
    parser.add_argument("command", nargs="?", help="The command to execute (optional)")
    
    args = parser.parse_args()
    
    if args.command:
        # Direct mode
        process_command(args.command)
    else:
        # Interactive mode
        repl()

if __name__ == "__main__":
    main()
