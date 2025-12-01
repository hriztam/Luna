# Luna Core

Luna Core is a Python-based CLI assistant designed for macOS automation. It allows users to control their system, open applications, and perform various tasks using natural language commands.

## Features

- **Natural Language Processing**: Understands user commands via LLM.
- **System Automation**: Can open apps, URLs, and run shell commands.
- **Extensible**: Modular design for adding new actions.

## Installation

1. Clone the repository.
2. Install dependencies:
   ```bash
   pip install -r requirements.txt
   pip install -e .
   ```

## Usage

Run the assistant using the CLI command:

```bash
luna
```

## Debugging

To enable detailed debug logging, set the `LUNA_DEBUG` environment variable to `1`:

```bash
export LUNA_DEBUG=1
luna "open safari"
```

Logs will be written to `~/.luna_debug.log` and include:

- User input and LLM responses
- Parsed actions and execution details
- Timing information and error tracebacks
