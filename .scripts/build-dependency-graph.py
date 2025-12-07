#!/usr/bin/env python3
"""
TaskMaster Dependency Graph Builder

Parses TaskMaster tasks.json and builds execution levels for parallel task execution.
Tasks at the same level have no dependencies on each other and can run in parallel.

Input: tasks.json from TaskMaster
Output: JSON array of execution levels [[1,3], [2], [4]]

Example:
    Task 1: dependencies=[]        -> Level 0
    Task 2: dependencies=[1]       -> Level 1  
    Task 3: dependencies=[]        -> Level 0 (parallel with Task 1)
    Task 4: dependencies=[1,2]     -> Level 2
"""

import json
import sys
from typing import Dict, List, Set


def parse_tasks(tasks_data: List[Dict]) -> Dict[str, List[str]]:
    """
    Parse tasks.json and extract task IDs with their dependencies.
    
    Args:
        tasks_data: List of task objects from tasks.json
        
    Returns:
        Dict mapping task_id -> list of dependency task_ids
        
    Example:
        Input: [{"id": 1, "dependencies": []}, {"id": 2, "dependencies": [1]}]
        Output: {"1": [], "2": ["1"]}
    """
    task_deps = {}
    
    for task in tasks_data:
        task_id = str(task.get("id", ""))
        if not task_id:
            continue
            
        # Get dependencies, handle both array and comma-separated string
        deps = task.get("dependencies", [])
        if isinstance(deps, str):
            deps = [d.strip() for d in deps.split(",") if d.strip()]
        
        # Normalize all dependency IDs to strings
        dep_ids = [str(d) for d in deps if d]
        
        task_deps[task_id] = dep_ids
    
    return task_deps


def validate_dependencies(task_deps: Dict[str, List[str]]) -> None:
    """
    Validate that all dependencies reference existing tasks.
    Raises ValueError if invalid dependencies found.
    """
    all_task_ids = set(task_deps.keys())
    
    for task_id, deps in task_deps.items():
        for dep in deps:
            if dep not in all_task_ids:
                raise ValueError(
                    f"Task {task_id} depends on non-existent task {dep}"
                )


def detect_circular_dependencies(task_deps: Dict[str, List[str]]) -> None:
    """
    Detect circular dependencies using depth-first search.
    Raises ValueError if circular dependency detected.
    """
    # Track visited nodes and current path
    visited = set()
    path = set()
    
    def dfs(task_id: str) -> bool:
        """Returns True if cycle detected"""
        if task_id in path:
            return True  # Cycle found
        if task_id in visited:
            return False  # Already processed, no cycle
            
        visited.add(task_id)
        path.add(task_id)
        
        for dep in task_deps.get(task_id, []):
            if dfs(dep):
                return True
                
        path.remove(task_id)
        return False
    
    for task_id in task_deps:
        if task_id not in visited:
            if dfs(task_id):
                raise ValueError(
                    f"Circular dependency detected involving task {task_id}"
                )


def build_execution_levels(task_deps: Dict[str, List[str]]) -> List[List[int]]:
    """
    Build execution levels from dependency graph.
    Tasks in the same level can run in parallel.
    
    Algorithm:
        1. Find all tasks with no dependencies (Level 0)
        2. Mark them as completed
        3. Find all tasks whose dependencies are all completed (Level 1)
        4. Repeat until all tasks are assigned
        
    Args:
        task_deps: Dict mapping task_id -> list of dependency task_ids
        
    Returns:
        List of levels, where each level is a list of task IDs
        
    Example:
        Input: {"1": [], "2": ["1"], "3": [], "4": ["1", "2"]}
        Output: [[1, 3], [2], [4]]
    """
    levels = []
    completed = set()
    remaining = set(task_deps.keys())
    
    max_iterations = len(task_deps) + 1  # Safety limit
    iteration = 0
    
    while remaining and iteration < max_iterations:
        current_level = []
        tasks_to_complete = []
        
        for task_id in list(remaining):
            deps = task_deps[task_id]
            
            # Can execute if all dependencies are completed
            if all(dep in completed for dep in deps):
                current_level.append(int(task_id))
                tasks_to_complete.append(task_id)
        
        if not current_level:
            # No tasks could be scheduled - shouldn't happen if validation passed
            raise RuntimeError(
                f"Unable to schedule remaining tasks: {remaining}. "
                f"This indicates a dependency issue that wasn't caught during validation."
            )
        
        # Mark all tasks in this level as completed AFTER building the level
        # This prevents tasks in the same level from depending on each other
        for task_id in tasks_to_complete:
            completed.add(task_id)
            remaining.remove(task_id)
        
        levels.append(sorted(current_level))
        iteration += 1
    
    if remaining:
        raise RuntimeError(
            f"Failed to schedule all tasks after {iteration} iterations. "
            f"Remaining: {remaining}"
        )
    
    return levels


def calculate_parallelism_stats(levels: List[List[int]]) -> Dict:
    """Calculate statistics about parallelization opportunities."""
    total_tasks = sum(len(level) for level in levels)
    max_parallel = max(len(level) for level in levels) if levels else 0
    avg_parallel = sum(len(level) for level in levels) / len(levels) if levels else 0
    
    # Theoretical speedup assuming all tasks take equal time
    sequential_time = total_tasks
    parallel_time = len(levels)  # Each level adds one time unit
    theoretical_speedup = sequential_time / parallel_time if parallel_time > 0 else 1.0
    
    return {
        "total_tasks": total_tasks,
        "num_levels": len(levels),
        "max_parallel_tasks": max_parallel,
        "avg_parallel_tasks": round(avg_parallel, 2),
        "theoretical_speedup": round(theoretical_speedup, 2)
    }


def main():
    """Main entry point for the script."""
    if len(sys.argv) < 2:
        print("Usage: build-dependency-graph.py <tasks.json> [output.json]", file=sys.stderr)
        print("\nReads TaskMaster tasks.json and outputs execution levels.", file=sys.stderr)
        print("Output format: JSON array of levels, e.g., [[1,3], [2], [4]]", file=sys.stderr)
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_file = sys.argv[2] if len(sys.argv) > 2 else None
    
    try:
        # Read input
        with open(input_file, 'r') as f:
            data = json.load(f)
        
        # Handle both TaskMaster formats:
        # 1. New tagged format: {"master": {"tasks": [...], "metadata": {...}}}
        # 2. Old array format: [...]
        if isinstance(data, dict):
            # New tagged format - extract tasks from tag
            # Default to "master" tag if no other tags found
            tag_name = next(iter(data.keys())) if data else "master"
            
            if tag_name not in data or "tasks" not in data[tag_name]:
                raise ValueError(
                    f"Expected tagged format with tasks array. "
                    f"Got: {json.dumps(data, indent=2)[:200]}..."
                )
            
            tasks_data = data[tag_name]["tasks"]
            print(f"ℹ️  Using tasks from tag: {tag_name}", file=sys.stderr)
        elif isinstance(data, list):
            # Old array format (backwards compatibility)
            tasks_data = data
            print(f"ℹ️  Using legacy array format", file=sys.stderr)
        else:
            raise ValueError(
                f"tasks.json must contain either an array or tagged object. "
                f"Got: {type(data).__name__}"
            )
        
        # Parse and validate
        task_deps = parse_tasks(tasks_data)
        
        if not task_deps:
            print("⚠️  No tasks found in input file", file=sys.stderr)
            output_data = {
                "levels": [],
                "stats": {"total_tasks": 0, "num_levels": 0}
            }
        else:
            validate_dependencies(task_deps)
            detect_circular_dependencies(task_deps)
            
            # Build execution levels
            levels = build_execution_levels(task_deps)
            stats = calculate_parallelism_stats(levels)
            
            # Prepare output
            output_data = {
                "levels": levels,
                "stats": stats
            }
            
            # Print summary to stderr (so it doesn't interfere with JSON output)
            print(f"✅ Dependency graph built successfully", file=sys.stderr)
            print(f"📊 Total tasks: {stats['total_tasks']}", file=sys.stderr)
            print(f"📊 Execution levels: {stats['num_levels']}", file=sys.stderr)
            print(f"📊 Max parallel tasks: {stats['max_parallel_tasks']}", file=sys.stderr)
            print(f"📊 Theoretical speedup: {stats['theoretical_speedup']}x", file=sys.stderr)
            print(f"", file=sys.stderr)
            print(f"Execution plan:", file=sys.stderr)
            for i, level in enumerate(levels):
                print(f"  Level {i}: Tasks {level} (parallel)", file=sys.stderr)
        
        # Write output
        if output_file:
            with open(output_file, 'w') as f:
                json.dump(output_data, f, indent=2)
            print(f"💾 Output written to: {output_file}", file=sys.stderr)
        else:
            # Print JSON to stdout
            print(json.dumps(output_data, indent=2))
    
    except FileNotFoundError:
        print(f"❌ Error: File not found: {input_file}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"❌ Error: Invalid JSON in {input_file}: {e}", file=sys.stderr)
        sys.exit(1)
    except ValueError as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"❌ Unexpected error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()

