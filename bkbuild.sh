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

# Resolve a relative path to an absolute path
resolve_path() {
  echo "$(cd "$1" && pwd)"
}

# Check if the virtual environment path is valid
is_valid_venv_path() {
  local venv_path=$1
  echo $venv_path
  [ -d "$venv_path" ] && [ -f "$venv_path/bin/activate" ] && [ -d "$venv_path/lib" ]
}

# List all virtual environments in the current directory
list_virtual_envs() {
  echo "Searching for virtual environments in the current directory..."
  local venvs=$(find . -maxdepth 1 -type d -name ".*" -exec test -e "{}/bin/activate" \; -print)
  if [ -z "$venvs" ]; then
    echo "No virtual environments found."
  else
    echo "Virtual environments found:"
    echo "$venvs"
  fi
}

create_virtual_env() {
  while true; do
    read -p "Enter a name for the new virtual environment: " venv_name
    if [[ "$venv_name" != .* ]]; then
      venv_name=".$venv_name"
    fi

    if [ -d "$venv_name" ]; then
      echo "Error: A virtual environment with the name '$venv_name' already exists. Please choose a different name."
    else
      break
    fi
  done

  read -p "Enter the desired Python version for the virtual environment (>= 3.9): " python_version
  if command_exists "python$python_version"; then
    python -m venv .venv
    source .venv/bin/activate
    echo "Virtual environment (.venv) created and activated."
    pip install -r requirements-dev.txt
  else
    echo "Error: Python version $python_version is not installed."
    exit 1
  fi
}

# Check if the Python toolchain is installed
check_python_toolchain() {
  if [ -n "$VIRTUAL_ENV" ]; then
    echo "Already in a virtual environment: $VIRTUAL_ENV"
  else
    read -p "You are not in a virtual environment. Do you want to load an existing one or create a new one? (load/create/n): " choice
    if [ "$choice" == "load" ]; then
      list_virtual_envs
      read -p "Enter the path to the existing virtual environment: " venv_path
      if is_valid_venv_path "$venv_path"; then
        venv_path=$(resolve_path "$venv_path")
        echo "Activating virtual environment '$venv_path'..."
        source "$venv_path/bin/activate" || (echo "Error: Unable to activate the virtual environment." && exit 1)
        echo "Virtual environment '$venv_path' loaded."
      else
        echo "Error: Invalid virtual environment path."
        exit 1
      fi
    elif [ "$choice" == "create" ]; then
      create_virtual_env
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
      "dev-debug-shadow-map")
        echo "Building for development with write shadow map..."
        maturin develop --features debug-shadow-map
        ;;
      "dev-debug-sunlight-map")
        echo "Building for development with write sunlight map..."
        maturin develop --features debug-sunlight-map
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
