#!/usr/bin/env python3
"""Schema-aware no-credit LLM stub for basic-project Lobster intake smoke tests."""
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


def project_text(payload: dict) -> str:
    inp = payload.get("input") if isinstance(payload.get("input"), dict) else {}
    parts: list[str] = []
    for value in inp.values():
        if isinstance(value, str):
            parts.append(value)
        elif isinstance(value, (dict, list)):
            try:
                parts.append(json.dumps(value))
            except Exception:
                pass
    text = "\n".join(parts)
    return text[:200000]


def is_sigma(payload: dict) -> bool:
    text = project_text(payload).lower()
    return any(token in text for token in ("sigma-1", "rental", "inventory", "equipment", "av product", "av-product"))


def base_tasks(project: str = "rest") -> list[dict]:
    if project == "sigma":
        return [
            {
                "id": 1,
                "title": "Model AV rental inventory and product catalog data",
                "description": "Define equipment, category, availability, pricing, and media metadata needed for Sigma-1 rental browsing and quoting.",
                "agent": "nova",
                "stack": "typescript-database",
                "priority": "high",
                "dependencies": [],
                "acceptance_criteria": ["Inventory entities cover AV equipment rental needs", "Seed/import path preserves existing product metadata"],
                "files": ["src/server/db/schema.ts", "src/server/services/inventory.ts", "tests/inventory.test.ts"],
                "decision_points": [],
            },
            {
                "id": 2,
                "title": "Build rental quote cart and reservation API",
                "description": "Implement API endpoints for browsing products, managing a rental cart, estimating price, and submitting quote requests.",
                "agent": "bolt",
                "stack": "typescript-api",
                "priority": "high",
                "dependencies": [1],
                "acceptance_criteria": ["Cart operations validate quantity and date range", "Quote submission records customer and line items"],
                "files": ["src/server/api/rentals.ts", "src/server/services/quotes.ts", "tests/rental-api.test.ts"],
                "decision_points": [],
            },
            {
                "id": 3,
                "title": "Create customer-facing rental browser UI",
                "description": "Add searchable equipment listing, product detail, cart review, and quote request screens for the Sigma-1 rental experience.",
                "agent": "prism",
                "stack": "typescript-react",
                "priority": "medium",
                "dependencies": [1, 2],
                "acceptance_criteria": ["Users can search and filter AV rental products", "Quote cart can be submitted from the UI"],
                "files": ["src/app/rentals/page.tsx", "src/components/rentals/*", "tests/rental-ui.test.tsx"],
                "decision_points": [],
            },
        ]
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
    project = "sigma" if is_sigma(payload) else "rest"

    if action == "text":
        if project == "sigma":
            return "Sigma-1 smoke deliberation: build a rental inventory platform for AV equipment with product catalog modeling, quote-cart APIs, and a searchable customer rental browser. Keep the smoke output concise and implementation-oriented."
        return "Simple REST API smoke deliberation: build a FastAPI service with health and todo CRUD endpoints, use in-memory storage for the smoke test, and add pytest coverage."

    if schema.endswith("generated-task.schema.json"):
        tasks = inp.get("tasks") if isinstance(inp.get("tasks"), list) else None
        return with_subtasks(tasks or base_tasks(project))

    if schema.endswith("project-decision-points.schema.json"):
        return []

    if schema.endswith("quality-gate.schema.json"):
        return {"pass": True, "score": 9, "summary": "smoke stub pass", "blocking_issues": [], "warnings": []}

    if schema.endswith("complexity-analysis.schema.json"):
        tasks = inp.get("tasks") if isinstance(inp.get("tasks"), list) else base_tasks(project)
        return [
            {"task_id": task.get("id", i + 1), "complexity": "small", "risk": "low", "reasoning": f"{project} smoke fixture"}
            for i, task in enumerate(tasks)
        ]

    if schema.endswith("capability-analysis.schema.json"):
        if project == "sigma":
            return {"required_capabilities": [
                {"capability": "typescript", "priority": "required", "reason": "Sigma-1 UI/API implementation"},
                {"capability": "database", "priority": "required", "reason": "Rental inventory persistence"},
                {"capability": "testing", "priority": "required", "reason": "Smoke verification"},
            ]}
        return {"required_capabilities": [
            {"capability": "python", "priority": "required", "reason": "FastAPI service"},
            {"capability": "testing", "priority": "required", "reason": "pytest coverage"},
        ]}

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
            fh.write(json.dumps({"action": ns.action, "schema": payload.get("schema", ""), "project": "sigma" if is_sigma(payload) else "rest"}) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
