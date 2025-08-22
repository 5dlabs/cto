#!/bin/bash

# Script to fix Talos YAML configuration files

echo "Fixing Talos YAML configuration files..."

# Function to fix indentation (convert 4 spaces to 2 spaces)
fix_indentation() {
    local file="$1"
    if [[ -f "$file" ]]; then
        echo "Fixing indentation in $file..."
        # Convert 4-space indentation to 2-space indentation
        sed -i '' 's/^    /  /g' "$file"
        sed -i '' 's/^        /    /g' "$file"
        sed -i '' 's/^            /      /g' "$file"
        sed -i '' 's/^                /        /g' "$file"
        sed -i '' 's/^                    /          /g' "$file"
        sed -i '' 's/^                        /            /g' "$file"
        sed -i '' 's/^                            /              /g' "$file"
        sed -i '' 's/^                                /                /g' "$file"
        sed -i '' 's/^                                    /                  /g' "$file"
        sed -i '' 's/^                                        /                    /g' "$file"
    fi
}

# Function to remove trailing spaces
remove_trailing_spaces() {
    local file="$1"
    if [[ -f "$file" ]]; then
        sed -i '' 's/[[:space:]]*$//' "$file"
    fi
}

# Fix controlplane.yaml
if [[ -f "infra/talos/config/simple/controlplane.yaml" ]]; then
    echo "Fixing controlplane.yaml..."
    
    # Add document start marker
    if ! grep -q "^---" "infra/talos/config/simple/controlplane.yaml"; then
        sed -i '' '1i\
---
' "infra/talos/config/simple/controlplane.yaml"
    fi
    
    # Fix indentation
    fix_indentation "infra/talos/config/simple/controlplane.yaml"
    
    # Remove trailing spaces
    remove_trailing_spaces "infra/talos/config/simple/controlplane.yaml"
    
    # Add newline at end
    echo "" >> "infra/talos/config/simple/controlplane.yaml"
fi

# Fix worker.yaml
if [[ -f "infra/talos/config/simple/worker.yaml" ]]; then
    echo "Fixing worker.yaml..."
    
    # Add document start marker
    if ! grep -q "^---" "infra/talos/config/simple/worker.yaml"; then
        sed -i '' '1i\
---
' "infra/talos/config/simple/worker.yaml"
    fi
    
    # Remove trailing spaces
    remove_trailing_spaces "infra/talos/config/simple/worker.yaml"
    
    # Add newline at end
    echo "" >> "infra/talos/config/simple/worker.yaml"
fi

echo "Talos YAML configuration files fixed!"
