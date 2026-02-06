#!/bin/bash
# Add ## Agent section to all subtasks based on their Subagent Type

TASKS_DIR="/Users/jonathonfritz/clawd-ctoworkflow/cto/tests/cli-invocation/config/tasks"

for task in $(ls -d "$TASKS_DIR"/task-* 2>/dev/null | sort -V); do
  task_name=$(basename $task)
  subtasks_dir="$task/subtasks"
  
  if [[ -d "$subtasks_dir" ]]; then
    for st in $(ls -d "$subtasks_dir"/task-* 2>/dev/null | sort -V); do
      prompt_file="$st/prompt.md"
      if [[ -f "$prompt_file" ]]; then
        # Skip if already has ## Agent
        if grep -q "^## Agent$" "$prompt_file"; then
          continue
        fi
        
        # Get subtask info
        subtask_id=$(basename $st)
        title=$(head -1 "$prompt_file" | sed 's/^# //' | sed 's/^Subtask [0-9.]*: //')
        subagent_type=$(grep "^## Subagent Type" "$prompt_file" -A1 | tail -1 | tr -d ' ')
        
        # Generate agent name based on title and type
        # Convert title to lowercase, replace spaces with hyphens, take key words
        agent_name=""
        
        # Determine agent name based on content
        if echo "$title" | grep -qi "review"; then
          # Review tasks get -reviewer suffix
          domain=$(echo "$title" | sed 's/Review.*//i' | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | sed 's/-$//' | sed 's/^-//')
          if [[ -z "$domain" ]]; then
            domain=$(echo "$task_name" | sed 's/task-//')
          fi
          agent_name="${domain}-reviewer"
        elif echo "$title" | grep -qi "test"; then
          # Test tasks
          domain=$(echo "$title" | sed 's/.*test.*//i' | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | head -c 20)
          agent_name="test-agent"
        elif echo "$title" | grep -qi "deploy\|setup\|initialize\|create\|install"; then
          # Deployment/setup tasks
          # Extract the key technology/component
          tech=$(echo "$title" | grep -oiE '(PostgreSQL|MongoDB|Redis|Kafka|RabbitMQ|SeaweedFS|Prometheus|Grafana|NGINX|Electron|Expo|Next\.js|Rust|Go|Bun)' | head -1 | tr '[:upper:]' '[:lower:]')
          if [[ -n "$tech" ]]; then
            agent_name="${tech}-deployer"
          else
            # Generic deployer based on title
            key=$(echo "$title" | tr '[:upper:]' '[:lower:]' | sed 's/deploy //;s/setup //;s/initialize //;s/create //' | tr ' ' '-' | cut -d'-' -f1-2)
            agent_name="${key}-deployer"
          fi
        elif echo "$title" | grep -qi "implement\|build\|add"; then
          # Implementation tasks
          key=$(echo "$title" | tr '[:upper:]' '[:lower:]' | sed 's/implement //;s/build //;s/add //' | tr ' ' '-' | cut -d'-' -f1-2)
          agent_name="${key}-implementer"
        elif echo "$title" | grep -qi "configure\|config"; then
          # Configuration tasks
          key=$(echo "$title" | tr '[:upper:]' '[:lower:]' | sed 's/configure //;s/config //' | tr ' ' '-' | cut -d'-' -f1-2)
          agent_name="${key}-agent"
        else
          # Default: use subagent type
          key=$(echo "$title" | tr '[:upper:]' '[:lower:]' | tr ' ' '-' | cut -d'-' -f1-2)
          agent_name="${key}-${subagent_type:-agent}"
        fi
        
        # Clean up agent name
        agent_name=$(echo "$agent_name" | sed 's/--/-/g' | sed 's/-$//' | sed 's/^-//' | cut -c1-30)
        
        # If still empty, use generic
        if [[ -z "$agent_name" || "$agent_name" == "-" ]]; then
          agent_name="subtask-${subtask_id}-agent"
        fi
        
        # Insert ## Agent after ## Subagent Type line
        if grep -q "^## Subagent Type" "$prompt_file"; then
          # Insert after Subagent Type section
          sed -i '' "/^## Subagent Type/,/^##/{
            /^## Subagent Type/a\\
\\
## Agent\\
${agent_name}
          }" "$prompt_file" 2>/dev/null || \
          sed -i "/^## Subagent Type/,/^##/{
            /^## Subagent Type/a\\
\\
## Agent\\
${agent_name}
          }" "$prompt_file"
        else
          # Insert after ## Parent Task section
          sed -i '' "/^## Parent Task/,/^##/{
            /^## Parent Task/a\\
\\
## Agent\\
${agent_name}
          }" "$prompt_file" 2>/dev/null || \
          sed -i "/^## Parent Task/,/^##/{
            /^## Parent Task/a\\
\\
## Agent\\
${agent_name}
          }" "$prompt_file"
        fi
        
        echo "✅ $task_name/$subtask_id → $agent_name"
      fi
    done
  fi
done

echo ""
echo "Done! All subtasks now have agents."
