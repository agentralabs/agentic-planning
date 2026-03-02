from __future__ import annotations

import pytest

from agentic_planning import PlanningError, PlanningGraph, __version__


def test_version_semver() -> None:
    parts = __version__.split(".")
    assert len(parts) == 3


def test_graph_init(tmp_path: pytest.TempPathFactory) -> None:
    graph = PlanningGraph(str(tmp_path / "test.aplan"))
    assert graph.path.name == "test.aplan"


def test_missing_binary_raises(tmp_path: pytest.TempPathFactory) -> None:
    graph = PlanningGraph(str(tmp_path / "test.aplan"), binary="nonexistent-aplan-binary")
    with pytest.raises(PlanningError):
        graph._binary()
