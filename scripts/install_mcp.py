#!/usr/bin/env python3
"""Install Fleet MCP into a Claude Desktop or Cursor JSON configuration.

The script merges only the ``fleet`` server entry, creates a timestamp-free
backup before replacing an existing file, and never writes a real token unless
``--allow-token-write`` is explicitly supplied.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import shutil
import sys
import tempfile
from pathlib import Path
from urllib.parse import urlparse


PLACEHOLDER_TOKEN = "replace-with-read-only-token"
PLACEHOLDER_URL = "https://fleet.example.com"


def default_config_path(client: str) -> Path:
    system = platform.system()
    if client == "claude":
        if system == "Windows":
            return Path(os.environ.get("APPDATA", Path.home())) / "Claude" / "claude_desktop_config.json"
        if system == "Darwin":
            return Path.home() / "Library/Application Support/Claude/claude_desktop_config.json"
        return Path.home() / ".config/Claude/claude_desktop_config.json"

    # Cursor supports a global ~/.cursor/mcp.json and a workspace .cursor/mcp.json.
    return Path.home() / ".cursor" / "mcp.json"


def validate_url(value: str) -> str:
    parsed = urlparse(value.strip())
    if parsed.scheme not in {"http", "https"} or not parsed.hostname:
        raise ValueError("FLEET_SERVER_URL must be an http(s) URL with a host")
    if parsed.username or parsed.password or parsed.query or parsed.fragment:
        raise ValueError("FLEET_SERVER_URL must not contain credentials, query, or fragment")
    loopback = parsed.hostname in {"localhost", "127.0.0.1", "::1"}
    if parsed.scheme == "http" and not loopback:
        raise ValueError("http is only allowed for loopback development URLs")
    return value.strip().rstrip("/")


def build_entry(binary: str, url: str, token: str) -> dict[str, object]:
    return {
        "command": binary,
        "args": [],
        "env": {
            "FLEET_SERVER_URL": url,
            "FLEET_API_TOKEN": token,
            "FLEET_ALLOWED_TOOLS": "fleet_list_hosts,fleet_get_host,fleet_search_hosts,fleet_get_host_by_identifier,fleet_list_reports,fleet_get_report,fleet_list_policies,fleet_get_policy,fleet_list_software,fleet_list_vulnerabilities,fleet_get_cve,fleet_get_version",
        },
    }


def load_config(path: Path) -> dict[str, object]:
    if not path.exists():
        return {}
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"existing config is not valid JSON: {exc}") from exc
    if not isinstance(value, dict):
        raise ValueError("existing config root must be a JSON object")
    return value


def write_config(path: Path, config: dict[str, object], dry_run: bool) -> None:
    rendered = json.dumps(config, ensure_ascii=False, indent=2) + "\n"
    if dry_run:
        print(rendered, end="")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        shutil.copy2(path, path.with_suffix(path.suffix + ".bak"))
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as handle:
        handle.write(rendered)
        temporary = Path(handle.name)
    os.replace(temporary, path)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client", choices=("claude", "cursor"), required=True)
    parser.add_argument("--config", type=Path, help="override the detected JSON config path")
    parser.add_argument("--binary", default="fleet-mcp", help="absolute path or executable name")
    parser.add_argument("--url", default=os.environ.get("FLEET_SERVER_URL", PLACEHOLDER_URL))
    parser.add_argument("--token", default=os.environ.get("FLEET_API_TOKEN", PLACEHOLDER_TOKEN))
    parser.add_argument(
        "--allow-token-write",
        action="store_true",
        help="allow writing --token into the client config (otherwise a placeholder is used)",
    )
    parser.add_argument("--dry-run", action="store_true", help="print the merged JSON without writing")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        url = validate_url(args.url)
        path = (args.config or default_config_path(args.client)).expanduser()
        token = args.token if args.allow_token_write else PLACEHOLDER_TOKEN
        config = load_config(path)
        servers = config.setdefault("mcpServers", {})
        if not isinstance(servers, dict):
            raise ValueError("existing mcpServers value must be a JSON object")
        servers["fleet"] = build_entry(args.binary, url, token)
        write_config(path, config, args.dry_run)
    except (OSError, ValueError) as exc:
        print(f"fleet-mcp config install failed: {exc}", file=sys.stderr)
        return 1

    if not args.dry_run:
        print(f"Fleet MCP configuration written to {path}")
        if token == PLACEHOLDER_TOKEN:
            print("Set FLEET_API_TOKEN in the client config to a read-only token before starting the client.")
        else:
            print("Warning: the API token was written to the client config; protect that file.", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
