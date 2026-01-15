#!/usr/bin/env python3
"""
Neon Daemon - Basic Operations Example

Demonstrates common Neon Postgres operations using the FGP Neon daemon.
Requires:
  - Neon daemon running (`fgp start neon`)
  - NEON_API_KEY environment variable set
  - Database connection string configured
"""

import json
import socket
import uuid
from pathlib import Path

SOCKET_PATH = Path.home() / ".fgp/services/neon/daemon.sock"


def call_daemon(method: str, params: dict = None) -> dict:
    """Send a request to the Neon daemon and return the response."""
    request = {
        "id": str(uuid.uuid4()),
        "v": 1,
        "method": method,
        "params": params or {}
    }

    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as sock:
        sock.connect(str(SOCKET_PATH))
        sock.sendall((json.dumps(request) + "\n").encode())

        response = b""
        while True:
            chunk = sock.recv(4096)
            if not chunk:
                break
            response += chunk
            if b"\n" in response:
                break

        return json.loads(response.decode().strip())


def list_projects():
    """List all Neon projects."""
    print("\n🗄️ Neon Projects")
    print("-" * 40)

    result = call_daemon("neon.projects", {})

    if result.get("ok"):
        projects = result["result"].get("projects", [])
        if not projects:
            print("  No projects found")
        for project in projects:
            print(f"  • {project.get('name')} ({project.get('id')})")
            print(f"    Region: {project.get('region_id')}")
            print(f"    Created: {project.get('created_at', 'unknown')}")
            print()
    else:
        print(f"  ❌ Error: {result.get('error')}")


def list_branches(project_id: str):
    """List branches for a project."""
    print(f"\n🌿 Branches for project: {project_id}")
    print("-" * 40)

    result = call_daemon("neon.branches", {"project_id": project_id})

    if result.get("ok"):
        branches = result["result"].get("branches", [])
        for branch in branches:
            is_primary = "⭐ " if branch.get("primary") else "  "
            print(f"{is_primary}{branch.get('name')} ({branch.get('id')})")
            print(f"    State: {branch.get('state')}")
            print()
    else:
        print(f"  ❌ Error: {result.get('error')}")


def run_query(sql: str, project_id: str = None, branch: str = None):
    """Execute a SQL query."""
    print(f"\n💾 Executing SQL")
    print("-" * 40)
    print(f"  Query: {sql[:80]}{'...' if len(sql) > 80 else ''}")

    params = {"sql": sql}
    if project_id:
        params["project_id"] = project_id
    if branch:
        params["branch"] = branch

    result = call_daemon("neon.query", params)

    if result.get("ok"):
        rows = result["result"].get("rows", [])
        columns = result["result"].get("columns", [])

        print(f"\n  Columns: {', '.join(columns)}")
        print(f"  Rows returned: {len(rows)}")

        if rows:
            print("\n  Results:")
            for row in rows[:10]:  # Show first 10 rows
                print(f"    {row}")
            if len(rows) > 10:
                print(f"    ... and {len(rows) - 10} more rows")
    else:
        print(f"  ❌ Error: {result.get('error')}")


def describe_table(table_name: str, project_id: str = None):
    """Get table schema information."""
    print(f"\n📋 Schema for table: {table_name}")
    print("-" * 40)

    params = {"table": table_name}
    if project_id:
        params["project_id"] = project_id

    result = call_daemon("neon.describe", params)

    if result.get("ok"):
        columns = result["result"].get("columns", [])
        for col in columns:
            nullable = "NULL" if col.get("nullable") else "NOT NULL"
            print(f"  • {col.get('name')}: {col.get('type')} {nullable}")
    else:
        print(f"  ❌ Error: {result.get('error')}")


def list_tables(project_id: str = None):
    """List all tables in the database."""
    print("\n📊 Database Tables")
    print("-" * 40)

    params = {}
    if project_id:
        params["project_id"] = project_id

    result = call_daemon("neon.tables", params)

    if result.get("ok"):
        tables = result["result"].get("tables", [])
        if not tables:
            print("  No tables found")
        for table in tables:
            print(f"  • {table.get('schema', 'public')}.{table.get('name')}")
    else:
        print(f"  ❌ Error: {result.get('error')}")


if __name__ == "__main__":
    print("Neon Daemon Examples")
    print("=" * 40)

    # Check daemon health first
    health = call_daemon("health")
    if not health.get("ok"):
        print("❌ Neon daemon not running. Start with: fgp start neon")
        print("   Also ensure NEON_API_KEY is set")
        exit(1)

    print("✅ Neon daemon is healthy")

    # Run examples
    list_projects()

    # Uncomment with your project ID:
    # list_branches("your-project-id")
    # list_tables("your-project-id")

    # Example queries (use with caution):
    # run_query("SELECT version();")
    # run_query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public';")
    # describe_table("users")
