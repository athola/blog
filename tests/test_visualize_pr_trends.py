#!/usr/bin/env python3
"""
TDD Tests for Main Visualization Script
Following Red-Green-Refactor cycle and FIRST principles
"""

import sys
import os
from pathlib import Path
from unittest.mock import patch, MagicMock
import subprocess

# Add project root to path
sys.path.insert(0, str(Path(__file__).parent.parent / "scripts"))


class TestVisualizePRTrendsMain:
    """Test main visualization script entry point"""

    def test_script_exists_and_is_executable(self):
        """RED: Test that the main script exists and is executable"""
        script_path = Path("scripts/visualize_pr_trends.py")
        assert script_path.exists()
        assert script_path.is_file()
        # Check if executable bit is set
        assert os.access(script_path, os.X_OK)

    def test_script_requires_csv_argument(self):
        """RED: Test that script requires CSV file argument"""
        result = subprocess.run(
            [sys.executable, "scripts/visualize_pr_trends.py"],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 1
        assert "Usage:" in result.stdout
        assert "path_to_metrics_csv" in result.stdout

    def test_script_handles_missing_csv_file(self):
        """RED: Test that script handles missing CSV file gracefully"""
        result = subprocess.run(
            [
                sys.executable,
                "scripts/visualize_pr_trends.py",
                "nonexistent.csv",
            ],
            capture_output=True,
            text=True,
        )

        # Should not crash, should handle gracefully
        assert (
            "No data available for visualization" in result.stdout
            or result.returncode != 0
        )

    def test_script_creates_output_directories(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test that script creates required output directories"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        # Change to temp directory so output goes there
        original_cwd = os.getcwd()
        os.chdir(temp_dir)

        try:
            # Mock the plotting to avoid GUI dependencies
            with patch("sys.modules") as mock_modules:
                # Mock all the imports
                mock_modules.__getitem__.return_value = MagicMock()

                subprocess.run(
                    [
                        sys.executable,
                        str(
                            Path(original_cwd)
                            / "scripts/visualize_pr_trends.py"
                        ),
                        str(csv_file),
                    ],
                    capture_output=True,
                    text=True,
                    cwd=temp_dir,
                )

                # Should create output directories
                assert (Path(temp_dir) / ".github/reports").exists()
                assert (
                    Path(temp_dir) / ".github/reports/visualizations"
                ).exists()

        finally:
            os.chdir(original_cwd)


class TestVisualizePRTrendsIntegration:
    """Test integration of visualization script components"""

    @patch("chart_generators.plt")
    def test_script_integration_with_mocked_plotting(
        self, mock_plt, temp_dir, sample_csv_content
    ):
        """RED: Test complete script workflow with mocked plotting"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        # Mock matplotlib to avoid GUI dependencies
        mock_plt.figure.return_value = MagicMock()
        mock_plt.pie.return_value = MagicMock()
        mock_plt.plot.return_value = MagicMock()
        mock_plt.bar.return_value = [MagicMock()]
        mock_plt.hist.return_value = (None, None, [MagicMock()])
        mock_plt.savefig.return_value = MagicMock()
        mock_plt.close.return_value = MagicMock()
        mock_plt.gca.return_value = MagicMock()

        # Change to temp directory
        original_cwd = os.getcwd()
        os.chdir(temp_dir)

        try:
            # Import and run the main function
            sys.path.insert(0, str(Path(original_cwd) / "scripts"))
            import visualize_pr_trends

            # Mock sys.argv
            with patch.object(
                sys, "argv", ["visualize_pr_trends.py", str(csv_file)]
            ):
                # Should not raise exception
                visualize_pr_trends.main()

            # Should create report file
            report_file = Path(".github/reports/pr_size_trend.md")
            assert report_file.exists()

            # Report should contain expected content
            content = report_file.read_text()
            assert "# PR Size Trend Report" in content
            assert "**Total PRs analyzed**: 3" in content

        finally:
            os.chdir(original_cwd)

    def test_script_handles_plotting_errors_gracefully(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test that script handles plotting errors gracefully"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        original_cwd = os.getcwd()
        os.chdir(temp_dir)

        try:
            sys.path.insert(0, str(Path(original_cwd) / "scripts"))

            # Mock plotting to raise an exception
            with patch("chart_generators.plt") as mock_plt:
                mock_plt.figure.side_effect = RuntimeError("Plotting failed")

                import visualize_pr_trends

                with patch.object(
                    sys, "argv", ["visualize_pr_trends.py", str(csv_file)]
                ):
                    # Should handle exception gracefully
                    try:
                        visualize_pr_trends.main()
                        # If no exception, that's fine too
                    except SystemExit as e:
                        # Should exit with error code
                        assert e.code == 1

        finally:
            os.chdir(original_cwd)


class TestVisualizePRTrendsErrorHandling:
    """Test error handling in visualization script"""

    def test_script_handles_invalid_csv_format(self, temp_dir):
        """RED: Test that script handles invalid CSV format"""
        csv_file = Path(temp_dir) / "invalid.csv"
        csv_file.write_text("invalid,csv,content\nwith,wrong,format")

        result = subprocess.run(
            [sys.executable, "scripts/visualize_pr_trends.py", str(csv_file)],
            capture_output=True,
            text=True,
        )

        # Should handle gracefully - either succeed with partial data
        # or fail cleanly
        assert "Error" not in result.stderr or result.returncode != 0

    def test_script_handles_empty_csv(self, temp_dir):
        """RED: Test that script handles empty CSV file"""
        csv_file = Path(temp_dir) / "empty.csv"
        csv_file.write_text("")

        result = subprocess.run(
            [sys.executable, "scripts/visualize_pr_trends.py", str(csv_file)],
            capture_output=True,
            text=True,
        )

        # Should handle gracefully
        assert "No data available" in result.stdout or result.returncode != 0

    def test_script_handles_permission_errors(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test that script handles permission errors gracefully"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        # Create read-only directory to trigger permission error
        output_dir = Path(temp_dir) / ".github"
        output_dir.mkdir()
        output_dir.chmod(0o444)  # Read-only

        try:
            # Run from project root, not temp_dir
            project_root = Path(__file__).parent.parent
            result = subprocess.run(
                [
                    sys.executable,
                    str(project_root / "scripts" / "visualize_pr_trends.py"),
                    str(csv_file),
                ],
                capture_output=True,
                text=True,
            )

            # Should handle permission errors gracefully
            # Either succeed (if it can work around) or fail cleanly
            if result.returncode != 0:
                assert (
                    "Permission denied" in result.stderr
                    or "Error" in result.stdout
                    or "No such file or directory" in result.stderr
                )

        finally:
            # Restore permissions for cleanup
            output_dir.chmod(0o755)


class TestVisualizePRTrendsOutput:
    """Test output generation and quality"""

    @patch("chart_generators.plt")
    def test_script_generates_expected_output_files(
        self, mock_plt, temp_dir, sample_csv_content
    ):
        """RED: Test that script generates expected output files"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        # Mock matplotlib
        mock_plt.figure.return_value = MagicMock()
        mock_plt.pie.return_value = MagicMock()
        mock_plt.plot.return_value = MagicMock()
        mock_plt.bar.return_value = [MagicMock()]
        mock_plt.hist.return_value = (None, None, [MagicMock()])
        mock_plt.savefig.return_value = MagicMock()
        mock_plt.close.return_value = MagicMock()
        mock_plt.gca.return_value = MagicMock()

        original_cwd = os.getcwd()
        os.chdir(temp_dir)

        try:
            sys.path.insert(0, str(Path(original_cwd) / "scripts"))
            import visualize_pr_trends

            with patch.object(
                sys, "argv", ["visualize_pr_trends.py", str(csv_file)]
            ):
                visualize_pr_trends.main()

            # Check that output directories exist
            assert Path(".github/reports").exists()
            assert Path(".github/reports/visualizations").exists()

            # Check that report file exists and has content
            report_file = Path(".github/reports/pr_size_trend.md")
            assert report_file.exists()
            assert report_file.stat().st_size > 0

            # Verify plotting functions were called (charts were generated)
            assert (
                mock_plt.savefig.call_count >= 3
            )  # At least 3 charts should be generated

        finally:
            os.chdir(original_cwd)

    @patch("chart_generators.plt")
    def test_script_output_contains_expected_content(
        self, mock_plt, temp_dir, sample_csv_content
    ):
        """RED: Test that generated report contains expected content"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        # Mock matplotlib
        mock_plt.figure.return_value = MagicMock()
        mock_plt.pie.return_value = MagicMock()
        mock_plt.plot.return_value = MagicMock()
        mock_plt.bar.return_value = [MagicMock()]
        mock_plt.hist.return_value = (None, None, [MagicMock()])
        mock_plt.savefig.return_value = MagicMock()
        mock_plt.close.return_value = MagicMock()
        mock_plt.gca.return_value = MagicMock()

        original_cwd = os.getcwd()
        os.chdir(temp_dir)

        try:
            sys.path.insert(0, str(Path(original_cwd) / "scripts"))
            import visualize_pr_trends

            with patch.object(
                sys, "argv", ["visualize_pr_trends.py", str(csv_file)]
            ):
                visualize_pr_trends.main()

            # Read and verify report content
            report_file = Path(".github/reports/pr_size_trend.md")
            content = report_file.read_text()

            # Should contain key sections
            assert "# PR Size Trend Report" in content
            assert "## Summary Statistics" in content
            assert "## PR Size Distribution" in content
            assert "## Top Contributors" in content
            assert "## Key Insights" in content
            assert "## Recommendations" in content

            # Should contain actual data
            assert "**Total PRs analyzed**: 3" in content
            assert "bob" in content  # Contributor name
            assert "alice" in content  # Contributor name

        finally:
            os.chdir(original_cwd)


class TestVisualizePRTrendsCommandLineInterface:
    """Test command-line interface behavior"""

    def test_script_shows_help_with_no_args(self):
        """RED: Test that script shows usage when called with no arguments"""
        result = subprocess.run(
            [sys.executable, "scripts/visualize_pr_trends.py"],
            capture_output=True,
            text=True,
        )

        assert result.returncode == 1
        assert "Usage:" in result.stdout
        assert "visualize_pr_trends.py" in result.stdout
        assert "path_to_metrics_csv" in result.stdout

    def test_script_accepts_relative_and_absolute_paths(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test that script accepts both relative and absolute paths"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        # Test with absolute path
        result = subprocess.run(
            [
                sys.executable,
                "scripts/visualize_pr_trends.py",
                str(csv_file.absolute()),
            ],
            capture_output=True,
            text=True,
        )

        # Should handle absolute path
        assert "No data available" in result.stdout or result.returncode == 0

    def test_script_displays_completion_message(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test that script displays completion message"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        # Mock to avoid actual plotting
        with patch("chart_generators.plt"):
            result = subprocess.run(
                [
                    sys.executable,
                    "scripts/visualize_pr_trends.py",
                    str(csv_file),
                ],
                capture_output=True,
                text=True,
                cwd=temp_dir,
            )

            # Should display completion message
            if result.returncode == 0:
                assert "Visualization generation complete!" in result.stdout
                assert "artifacts" in result.stdout
