#!/bin/bash
# .git/hooks/pre-commit

# Exit on any error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "${YELLOW}🐍 Running Python pre-comm1it checks...${NC}"

# Function to check if Python files were modified
check_python_files() {
    git diff --cached --name-status | grep -E '^[AM].*\.py$' | cut -f2
}

# Get modified Python files
python_files=$(check_python_files)

# Only run checks if Python files were modified and exist
if [ -n "$python_files" ]; then
    # Filter to only existing files
    existing_files=""
    for file in $python_files; do
        if [ -f "$file" ]; then
            existing_files="$existing_files $file"
        fi
    done
    
    if [ -n "$existing_files" ]; then
        # Check virtual environment
#        if [ ! -d "venv" ] && [ ! -d "env" ]; then
#            echo "${RED}❌ Virtual environment not found. Please create one:${NC}"
#            echo "python -m venv venv"
#            exit 1
#        fi

        # Ensure dependencies are installed
        echo "📦 Checking dependencies..."
        pip install -q black flake8 pytest pytest-cov isort

        # Format code with Black
        echo "🎨 Formatting code with Black..."
        python -m black $existing_files

        # Sort imports with isort
        echo "📝 Sorting imports with isort..."
        python -m isort $existing_files

        # Run Flake8
        echo "🔍 Running Flake8..."
        if ! python -m flake8 $existing_files; then
            echo "${RED}❌ Flake8 check failed. Please fix the issues above.${NC}"
            exit 1
        fi

        # Run tests related to changed files
        echo "🧪 Running related tests..."
        if ! python -m pytest $existing_files -v; then
            echo "${RED}❌ Tests failed. Please fix the failing tests.${NC}"
            exit 1
        fi

        # Check for sensitive information
        echo "🔒 Checking for sensitive information..."
        if git diff --cached | grep -i "password\|secret\|api_key\|token"; then
            echo "${RED}❌ Warning: Possible sensitive information detected${NC}"
            exit 1
        fi
    fi
fi

echo "${GREEN}✅ All checks passed!${NC}"
