from setuptools import setup, find_packages

setup(
    name="luna-core",
    version="0.1.0",
    description="A Python-based CLI assistant for macOS automation",
    packages=find_packages(),
    install_requires=[
        "openai",
        "python-dotenv",
    ],
    entry_points={
        "console_scripts": [
            "luna=luna.cli:main",
        ],
    },
)
