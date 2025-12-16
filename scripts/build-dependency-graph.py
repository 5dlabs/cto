#!/usr/bin/env python3
"""
Build a dependency graph from tasks.json and output execution levels.

This script analyzes task dependencies and groups tasks into parallel execution levels.
Tasks with no unmet dependencies can run in parallel within the same level.

Usage:
    python3 build-dependency-graph.py <tasks.json> <output.json>

Output format:
{
    "levels": [["1", "2"], ["3"], ["4", "5"]],
    "stats": {
        "total_tasks": 5,
        "total_levels": 3,
        "max_parallelism": 2,
        "avg_parallelism": 1.67
    }
}
"""

import json
import sys
from collections import defaultdict


def load_tasks(tasks_file: str) -> list:
    """Load tasks from tasks.json file."""
    with open(tasks_file, 'r') as f:
        data = json.load(f)
    
    # Handle both array and object with 'tasks' key
    if isinstance(data, list):
        return data
    elif isinstance(data, dict) and 'tasks' in data:
        return data['tasks']
    else:
        raise ValueError("Invalid tasks.json format: expected array or object with 'tasks' key")


def build_dependency_graph(tasks: list) -> tuple:
    """
    Build dependency graph from tasks.
    
    Returns:
        - deps: dict mapping task_id -> set of dependency task_ids
        - task_lookup: dict mapping task_id -> task object
    """
    task_lookup = {}
    deps = defaultdict(set)
    
    for task in tasks:
        task_id = str(task.get('id', ''))
        if not task_id:
            continue
            
        task_lookup[task_id] = task
        
        # Get dependencies - could be 'dependencies' or 'dependsOn'
        dependencies = task.get('dependencies', []) or task.get('dependsOn', [])
        
        # Normalize dependencies to list of IDs
        for dep in dependencies:
            if isinstance(dep, dict):
                dep_id = str(dep.get('id', ''))
            else:
                dep_id = str(dep)
            
            if dep_id:
                deps[task_id].add(dep_id)
    
    return deps, task_lookup


def compute_execution_levels(deps: dict, task_lookup: dict) -> list:
    """
    Compute execution levels using topological sort.
    
    Tasks with no dependencies go to level 0.
    Tasks whose dependencies are all in earlier levels go to the next level.
    """
    if not task_lookup:
        return []
    
    # Track which level each task belongs to
    task_levels = {}
    remaining_tasks = set(task_lookup.keys())
    
    levels = []
    
    while remaining_tasks:
        # Find tasks whose dependencies are all resolved
        ready_tasks = []
        
        for task_id in remaining_tasks:
            task_deps = deps.get(task_id, set())
            # Check if all dependencies are in earlier levels
            unmet_deps = task_deps - set(task_levels.keys())
            
            if not unmet_deps:
                ready_tasks.append(task_id)
        
        if not ready_tasks:
            # Circular dependency detected - add remaining tasks to final level
            print(f"Warning: Circular or unresolvable dependencies detected for: {remaining_tasks}", file=sys.stderr)
            ready_tasks = list(remaining_tasks)
        
        # Sort tasks by ID for consistent ordering
        ready_tasks.sort(key=lambda x: int(x) if x.isdigit() else x)
        
        # Add to current level
        levels.append(ready_tasks)
        
        # Mark these tasks as resolved
        for task_id in ready_tasks:
            task_levels[task_id] = len(levels) - 1
            remaining_tasks.discard(task_id)
    
    return levels


def compute_stats(levels: list) -> dict:
    """Compute parallelism statistics."""
    if not levels:
        return {
            "total_tasks": 0,
            "total_levels": 0,
            "max_parallelism": 0,
            "avg_parallelism": 0.0
        }
    
    total_tasks = sum(len(level) for level in levels)
    level_sizes = [len(level) for level in levels]
    
    return {
        "total_tasks": total_tasks,
        "total_levels": len(levels),
        "max_parallelism": max(level_sizes) if level_sizes else 0,
        "avg_parallelism": round(total_tasks / len(levels), 2) if levels else 0.0
    }


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <tasks.json> <output.json>", file=sys.stderr)
        sys.exit(1)
    
    tasks_file = sys.argv[1]
    output_file = sys.argv[2]
    
    try:
        # Load and process tasks
        tasks = load_tasks(tasks_file)
        print(f"📊 Loaded {len(tasks)} tasks from {tasks_file}")
        
        # Build dependency graph
        deps, task_lookup = build_dependency_graph(tasks)
        print(f"🔗 Built dependency graph with {len(deps)} dependency relationships")
        
        # Compute execution levels
        levels = compute_execution_levels(deps, task_lookup)
        print(f"📈 Computed {len(levels)} execution levels")
        
        # Compute statistics
        stats = compute_stats(levels)
        
        # Write output
        output = {
            "levels": levels,
            "stats": stats
        }
        
        with open(output_file, 'w') as f:
            json.dump(output, f, indent=2)
        
        print(f"✅ Output written to {output_file}")
        
        # Print summary
        print(f"\n📊 Summary:")
        print(f"   Total tasks: {stats['total_tasks']}")
        print(f"   Execution levels: {stats['total_levels']}")
        print(f"   Max parallelism: {stats['max_parallelism']}")
        print(f"   Avg parallelism: {stats['avg_parallelism']}")
        
        for i, level in enumerate(levels):
            print(f"\n   Level {i}: {level}")
        
    except FileNotFoundError:
        print(f"❌ File not found: {tasks_file}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"❌ Invalid JSON in {tasks_file}: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
