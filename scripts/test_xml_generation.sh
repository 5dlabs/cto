#!/bin/bash

# Test script to verify XML generation fixes

echo "Testing XML generation fixes..."

# Test data with special characters
test_details="Technical Requirements:
- Handle XML & HTML entities properly
- Support <tags> and \"quotes\"
- Process 'apostrophes' correctly
- Version 1.0 should work"

test_strategy="1. Implementation must handle special characters like &, <, >, \", '
2. Code should be compatible with version 2.1 and later
3. All XML output should be properly formatted"

echo "Input details: $test_details"
echo "Input test strategy: $test_strategy"
echo ""

# Simulate the XML generation logic

# Extract technical specifications from task details
specs=""
if echo "$test_details" | grep -A 10 -i "technical\|requirements\|specifications" >/dev/null 2>&1; then
  specs_section=$(echo "$test_details" | sed -n '/technical\|requirements\|specifications/I,/^$/p' | head -10)
  while IFS= read -r line; do
    # Case-insensitive check to exclude section headers
    if [[ -n "$line" ]] && ! echo "$line" | grep -qi "^[[:space:]]*\(technical\|requirements\|specifications\)[[:space:]]*$"; then
      # XML escape the content
      escaped_line=$(echo "$line" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g; s/'\''/\&#39;/g')
      specs="${specs}<spec>$escaped_line</spec>
"
    fi
  done <<< "$specs_section"
fi

# Fallback specs if none found
if [ -z "$specs" ]; then
  specs="<spec>Follow the project&apos;s existing architecture and coding patterns</spec>
<spec>Ensure compatibility with the current technology stack</spec>"
fi

# Extract acceptance criteria from test strategy
criteria=""
if [ -n "$test_strategy" ]; then
  # Split test strategy into individual criteria on numbered items (1., 2., etc.)
  # Use word boundaries to avoid splitting on version numbers or decimals
  IFS=$'\n' read -rd '' -a test_parts <<< "$(echo "$test_strategy" | grep -E '^[[:space:]]*[0-9]+[[:space:]]*\.[[:space:]]*[A-Z]')"
  for part in "${test_parts[@]}"; do
    part=$(echo "$part" | sed 's/^[[:space:]]*[0-9]\+\.[[:space:]]*//;s/[[:space:]]*$//')
    if [ -n "$part" ] && [ ${#part} -gt 10 ]; then
      # XML escape the content
      escaped_part=$(echo "$part" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g; s/'\''/\&#39;/g')
      criteria="${criteria}<criterion>$escaped_part</criterion>
"
    fi
  done
fi

# Fallback criteria if none extracted
if [ -z "$criteria" ]; then
  criteria="<criterion>Implementation must be complete and functional</criterion>
<criterion>Code must follow the project&apos;s style guidelines and conventions</criterion>"
fi

# Generate XML
cat << XML_EOF
<prompt>
  <task>
      <id>test-123</id>
      <title>Test Task</title>
      <description>Test task with special characters</description>
  </task>
  <technical_specifications>
      $specs
  </technical_specifications>
  <acceptance_criteria>
      $criteria
  </acceptance_criteria>
</prompt>
XML_EOF

echo ""
echo "XML generation test completed!"
echo "Check the output above for proper XML escaping and formatting."
