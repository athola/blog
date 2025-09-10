#!/usr/bin/env python3
"""
pytest configuration file - TDD Foundation

Follows FIRST principles: Fast, Independent, Repeatable,
Self-validating, Timely
"""

import pytest
import tempfile
import sys
from pathlib import Path

# Add project root to Python path for imports
PROJECT_ROOT = Path(__file__).parent.parent
sys.path.insert(0, str(PROJECT_ROOT / "scripts"))


@pytest.fixture
def temp_dir():
    """Create temporary directory for test isolation."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        yield tmp_dir


@pytest.fixture
def sample_pr_data():
    """Provide sample PR data for testing."""
    return [
        {
            "date": "2024-01-15T10:30:00Z",
            "pr_number": 101,
            "title": "Add user authentication",
            "author": "alice",
            "total_lines": 450,
            "additions": 380,
            "deletions": 70,
            "changed_files": 8,
            "category": "ideal",
            "url": "https://github.com/example/repo/pull/101",
        },
        {
            "date": "2024-01-16T14:22:00Z",
            "pr_number": 102,
            "title": "Refactor database queries",
            "author": "bob",
            "total_lines": 850,
            "additions": 520,
            "deletions": 330,
            "changed_files": 12,
            "category": "good",
            "url": "https://github.com/example/repo/pull/102",
        },
        {
            "date": "2024-01-20T13:20:00Z",
            "pr_number": 106,
            "title": "Add comprehensive testing",
            "author": "bob",
            "total_lines": 2200,
            "additions": 2000,
            "deletions": 200,
            "changed_files": 45,
            "category": "too-large",
            "url": "https://github.com/example/repo/pull/106",
        },
    ]


@pytest.fixture
def sample_csv_content():
    """Provide sample CSV content for testing."""
    return (
        "date,pr_number,title,author,total_lines,additions,deletions,"
        "changed_files,category,url\n"
        "2024-01-15T10:30:00Z,101,Add user authentication,alice,450,380,70,"
        "8,ideal,https://github.com/example/repo/pull/101\n"
        "2024-01-16T14:22:00Z,102,Refactor database queries,bob,850,520,"
        "330,12,good,https://github.com/example/repo/pull/102\n"
        "2024-01-20T13:20:00Z,106,Add comprehensive testing,bob,2200,2000,"
        "200,45,too-large,https://github.com/example/repo/pull/106"
    )


@pytest.fixture
def mock_config():
    """Provide mock configuration for testing."""
    return {
        "MAX_PR_SIZE": 2000,
        "IDEAL_PR_SIZE": 500,
        "GOOD_PR_SIZE": 1500
    }


@pytest.fixture
def git_repo_structure(temp_dir):
    """Create mock git repository structure for testing."""
    repo_dir = Path(temp_dir) / "test_repo"
    repo_dir.mkdir()

    # Create mock files
    (repo_dir / "src").mkdir()
    (repo_dir / "src" / "main.py").write_text("print('hello')")
    (repo_dir / "tests").mkdir()
    (repo_dir / "tests" / "test_main.py").write_text("def test_main(): pass")
    (repo_dir / "README.md").write_text("# Test Repo")

    return repo_dir
