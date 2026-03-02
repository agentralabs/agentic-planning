"""AgenticPlanning Python SDK.

Wraps the `aplan` CLI for lightweight scripting.
"""

from __future__ import annotations

import json
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any

__version__ = "0.1.0"


class PlanningError(Exception):
    """Raised when an `aplan` command fails."""


@dataclass
class PlanningGraph:
    path: str | Path
    binary: str = "aplan"

    def __post_init__(self) -> None:
        self.path = Path(self.path)

    def _binary(self) -> str:
        found = shutil.which(self.binary)
        if not found:
            raise PlanningError(
                f"Cannot find '{self.binary}' on PATH. Install via https://agentralabs.tech/install/planning"
            )
        return found

    def _run_json(self, *args: str) -> Any:
        cmd = [self._binary(), "--file", str(self.path), *args]
        result = subprocess.run(cmd, capture_output=True, text=True)
        if result.returncode != 0:
            raise PlanningError(result.stderr.strip() or "aplan command failed")
        raw = result.stdout.strip()
        return json.loads(raw) if raw else {}

    def create_goal(self, title: str, intention: str) -> Any:
        return self._run_json("goal", "create", title, "--intention", intention)

    def list_goals(self) -> Any:
        return self._run_json("goal", "list")

    @property
    def exists(self) -> bool:
        return self.path.exists()
