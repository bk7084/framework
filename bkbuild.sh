#!/bin/bash

# Check if a command exists
command_exists() {
  command -v "$1" >/dev/null 2>&1
}

# Check if the Rust toolchain is installed
check_rust_toolchain() {
  # Check if cargo is installed
  if ! command_exists "cargo"; then
    echo "Error: Cargo is not installed."
    exit 1
  fi
  # Check if the Rust toolchain is installed
  local rust_version=$(rustc --version | cut -d' ' -f2)
  if [ "$(printf '%s\n' "1.74" "$rust_version" | sort -V | head -n1)" != "1.74" ]; then
      echo "Error: Rust version should be >= 1.74"
      exit 1
  fi
}

# Check if the virtual environment path is valid
is_valid_venv_path() {
  local venv_path=$1
  [ -d "$venv_path" ] && [ -f "$venv_path/bin/activate" ] && [ -d "$venv_path/lib" ]
}

# Check if the Python toolchain is installed
check_python_toolchain() {
  if [ -n "$VIRTUAL_ENV" ]; then
    echo "Already in a virtual environment."
  else
    read -p "You are not in a virtual environment. Do you want to load an existing one or create a new one? (load/create/n): " choice
    if [ "$choice" == "load" ]; then
      read -p "Enter the path to the existing virtual environment: " venv_path
      if is_valid_venv_path "$venv_path"; then
        source "$venv_path/bin/activate" || (echo "Error: Unable to activate the virtual environment." && exit 1)
        echo "Virtual environment loaded."
      else
        echo "Error: Invalid virtual environment path."
        exit 1
      fi
    elif [ "$choice" == "create" ]; then
      read -p "Enter the desired Python version for the virtual environment (>= 3.9): " python_version
      if command_exists "python$python_version"; then
        python -m venv .venv
        source .venv/bin/activate
        echo "Virtual environment created and activated."
        pip install -r requirements-dev.txt
      else
        echo "Error: Python version $python_version is not installed."
        exit 1
      fi
    else
      echo "You need to create or load the virtual environment yourself."
      exit 1
    fi
  fi
}

check_rust_toolchain
check_python_toolchain

# Check if the correct number of arguments is provided
if [ "$#" -lt 1 ]; then
  echo "Usage: $0 {run|build|pkg|pub-test} [debug|trace]"
  exit 1
fi

# Parse the command
command=$1
subcommand=$2

# Perform actions based on the command
case $command in
  "run")
    case $subcommand in
      "debug")
        echo "Running with debug mode..."
        # Add your debug-specific commands here
        ;;
      "trace")
        echo "Running with trace mode..."
        # Add your trace-specific commands here
        ;;
      *)
        echo "Running..."
        # Add your default run commands here
        ;;
    esac
    ;;
  "build")
    case $subcommand in
      "dev")
        echo "Building for development..."
        maturin develop
        ;;
      "rel")
        echo "Building for release..."
        maturin develop --release
        ;;
      *) echo "No subcommands provided... {dev|rel}"
        # Add your default build commands here
        ;;
    esac
    ;;
  "pkg")
    echo "Packaging..."
    maturin build --release
    ;;
  "pub-test")
    echo "Publishing to test.pypi.org..."
    maturin publish --repository testpypi
    ;;
  *)
    echo "Invalid command: $command"
    echo "Usage: $0 {run|build|pkg} [debug|trace]"
    exit 1
    ;;
esac

# Add any common commands that should be executed for all cases here

echo "Done."
