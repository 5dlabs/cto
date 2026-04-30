#!/usr/bin/env python3
"""Schema-aware no-credit LLM stub for simple REST API Lobster intake smoke tests."""
from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path


def load_payload(ns: argparse.Namespace) -> dict:
    raw = "{}"
    if ns.args_file:
        raw = Path(ns.args_file).read_text()
    elif ns.args_json:
        raw = ns.args_json
    try:
        return json.loads(raw or "{}")
    except Exception:
        return {}


def base_tasks() -> list[dict]:
    return [
        {
            "id": 1,
            "title": "Create FastAPI application scaffold",
            "description": "Set up a minimal FastAPI service with project metadata, dependency management, and a health endpoint.",
            "agent": "bolt",
            "stack": "python-fastapi",
            "priority": "high",
            "dependencies": [],
            "acceptance_criteria": ["GET /health returns 200", "Application can be started locally"],
            "files": ["pyproject.toml", "app/main.py", "tests/test_health.py"],
            "decision_points": [],
        },
        {
            "id": 2,
            "title": "Implement todo CRUD REST endpoints",
            "description": "Add in-memory todo create, list, get, update, and delete endpoints with validation.",
            "agent": "nova",
            "stack": "python-fastapi",
            "priority": "high",
            "dependencies": [1],
            "acceptance_criteria": ["CRUD endpoints follow REST conventions", "Invalid payloads return 422"],
            "files": ["app/routes/todos.py", "app/models.py", "tests/test_todos.py"],
            "decision_points": [],
        },
    ]


def with_subtasks(tasks: list[dict]) -> list[dict]:
    enriched = []
    for task in tasks:
        tid = int(task.get("id", len(enriched) + 1))
        item = dict(task)
        item["subtasks"] = [
            {
                "id": tid * 10 + 1,
                "title": f"Write tests for {task['title']}",
                "description": "Create behavior-focused tests before implementation.",
                "dependencies": [],
            },
            {
                "id": tid * 10 + 2,
                "title": f"Implement {task['title']}",
                "description": "Implement the minimal code needed to satisfy the tests.",
                "dependencies": [tid * 10 + 1],
            },
        ]
        enriched.append(item)
    return enriched


def output_for(payload: dict, action: str) -> object:
    schema = str(payload.get("schema", ""))
    inp = payload.get("input") if isinstance(payload.get("input"), dict) else {}

    if action == "text":
        return "Simple REST API smoke deliberation: build a FastAPI service with health and todo CRUD endpoints, use in-memory storage for the smoke test, and add pytest coverage."

    if schema.endswith("generated-task.schema.json"):
        tasks = inp.get("tasks") if isinstance(inp.get("tasks"), list) else None
        return with_subtasks(tasks or base_tasks())

    if schema.endswith("project-decision-points.schema.json"):
        return []

    if schema.endswith("quality-gate.schema.json"):
        return {"pass": True, "score": 9, "summary": "smoke stub pass", "blocking_issues": [], "warnings": []}

    if schema.endswith("complexity-analysis.schema.json"):
        tasks = inp.get("tasks") if isinstance(inp.get("tasks"), list) else base_tasks()
        return [
            {"task_id": task.get("id", i + 1), "complexity": "small", "risk": "low", "reasoning": "simple REST API smoke fixture"}
            for i, task in enumerate(tasks)
        ]

    if schema.endswith("capability-analysis.schema.json"):
        return {"capabilities": ["fastapi", "pytest", "rest-api"], "gaps": [], "recommendations": []}

    if schema.endswith("skill-recommendations.schema.json"):
        return {"recommendations": [], "skills": [], "missing_skills": []}

    if schema.endswith("tool-manifest.schema.json"):
        return {"tools": [], "manifest": [], "recommendations": []}

    if schema.endswith("skill-generation-result.schema.json"):
        return {"generated": [], "skipped": True, "reason": "smoke stub"}

    if schema.endswith("catalog-query-result.schema.json"):
        return {"matches": [], "queries": []}

    if schema.endswith("scale-tasks.schema.json"):
        return {"tasks": [], "recommendations": []}

    if schema.endswith("security-report.schema.json"):
        return {"risk_level": "low", "findings": [], "summary": "smoke stub"}

    if schema.endswith("remediation-tasks.schema.json"):
        return []

    if schema.endswith("scaffold.schema.json"):
        return {"files": [], "commands": [], "notes": "smoke stub"}

    return {"ok": True, "schema": schema, "smoke": True}


def main() -> int:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--tool")
    parser.add_argument("--action", default="json")
    parser.add_argument("--args-json")
    parser.add_argument("--args-file")
    ns, _ = parser.parse_known_args()
    payload = load_payload(ns)
    result = output_for(payload, ns.action)
    if isinstance(result, str):
        print(result)
    else:
        print(json.dumps(result))
    log = os.environ.get("SIMPLE_REST_SMOKE_LLM_LOG")
    if log:
        with open(log, "a") as fh:
            fh.write(json.dumps({"action": ns.action, "schema": payload.get("schema", "")}) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
