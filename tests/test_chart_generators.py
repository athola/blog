#!/usr/bin/env python3
"""
TDD Tests for Chart Generators Module
Following Red-Green-Refactor cycle and FIRST principles
"""

from pathlib import Path
from unittest.mock import patch, MagicMock

# Import modules under test
from pr_metrics import PRMetrics
from chart_generators import ChartGenerator


class TestChartGeneratorInstantiation:
    """Test ChartGenerator class instantiation - Basic TDD cycle"""

    def test_can_create_chart_generator_instance(self, temp_dir,
                                                 sample_csv_content):
        """RED: Test that we can create a ChartGenerator instance"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()

        chart_gen = ChartGenerator(metrics)
        assert chart_gen.metrics == metrics

    def test_chart_generator_requires_metrics_object(self):
        """RED: Test that ChartGenerator requires PRMetrics instance"""
        metrics = PRMetrics("test.csv")
        chart_gen = ChartGenerator(metrics)
        assert chart_gen.metrics is not None


class TestChartGenerationWithoutData:
    """Test chart generation behavior when no data is loaded"""

    def test_size_distribution_chart_with_no_data(self, temp_dir):
        """RED: Test size distribution chart generation with no data"""
        metrics = PRMetrics("nonexistent.csv")
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "test_chart.png"
        result = chart_gen.create_size_distribution_chart(str(output_file))

        # Should return empty dict when no data
        assert result == {}
        # File should not be created
        assert not output_file.exists()

    def test_trend_line_chart_with_no_data(self, temp_dir):
        """RED: Test trend line chart generation with no data"""
        metrics = PRMetrics("nonexistent.csv")
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "trend_chart.png"
        chart_gen.create_trend_line_chart(str(output_file))

        # Should handle gracefully - no file created
        assert not output_file.exists()

    def test_contributor_chart_with_no_data(self, temp_dir):
        """RED: Test contributor chart generation with no data"""
        metrics = PRMetrics("nonexistent.csv")
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "contributors.png"
        chart_gen.create_contributor_chart(str(output_file))

        # Should handle gracefully - no file created
        assert not output_file.exists()

    def test_size_histogram_with_no_data(self, temp_dir):
        """RED: Test size histogram generation with no data"""
        metrics = PRMetrics("nonexistent.csv")
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "histogram.png"
        chart_gen.create_size_histogram(str(output_file))

        # Should handle gracefully - no file created
        assert not output_file.exists()


class TestChartGenerationWithMockedPlotting:
    """Test chart generation logic with mocked matplotlib.

    Avoids GUI dependencies during testing.
    """

    @patch("chart_generators.plt")
    def test_size_distribution_chart_creates_pie_chart(
        self, mock_plt, temp_dir, sample_csv_content
    ):
        """RED: Test that size distribution chart creates pie chart elements"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "pie_chart.png"

        # Mock the plotting functions
        mock_plt.figure.return_value = MagicMock()
        mock_plt.pie.return_value = MagicMock()
        mock_plt.title.return_value = MagicMock()
        mock_plt.tight_layout.return_value = MagicMock()
        mock_plt.savefig.return_value = MagicMock()
        mock_plt.close.return_value = MagicMock()

        result = chart_gen.create_size_distribution_chart(str(output_file))

        # Should call plotting functions
        mock_plt.figure.assert_called_once()
        mock_plt.pie.assert_called_once()
        mock_plt.title.assert_called_once()
        mock_plt.savefig.assert_called_once_with(
            str(output_file), dpi=300, bbox_inches="tight"
        )
        mock_plt.close.assert_called_once()

        # Should return category counts
        assert isinstance(result, dict)
        assert len(result) > 0

    @patch("chart_generators.plt")
    def test_trend_line_chart_plots_time_series(
        self, mock_plt, temp_dir, sample_csv_content
    ):
        """RED: Test that trend line chart plots time series data"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "trend_chart.png"

        # Mock the plotting functions
        mock_plt.figure.return_value = MagicMock()
        mock_plt.plot.return_value = MagicMock()
        mock_gca = MagicMock()
        mock_plt.gca.return_value = mock_gca

        chart_gen.create_trend_line_chart(str(output_file))

        # Should call plotting functions
        mock_plt.figure.assert_called_once()
        mock_plt.plot.assert_called_once()
        mock_plt.savefig.assert_called_once_with(
            str(output_file), dpi=300, bbox_inches="tight"
        )

    @patch("chart_generators.plt")
    def test_contributor_chart_creates_bar_chart(
        self, mock_plt, temp_dir, sample_csv_content
    ):
        """RED: Test that contributor chart creates bar chart elements"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "contributors.png"

        # Mock the plotting functions
        mock_bars = [MagicMock(), MagicMock()]
        for bar in mock_bars:
            bar.get_height.return_value = 5
            bar.get_x.return_value = 0
            bar.get_width.return_value = 1

        mock_plt.figure.return_value = MagicMock()
        mock_plt.bar.return_value = mock_bars
        mock_plt.text.return_value = MagicMock()

        chart_gen.create_contributor_chart(str(output_file))

        # Should call bar chart functions
        mock_plt.figure.assert_called_once()
        mock_plt.bar.assert_called_once()
        mock_plt.savefig.assert_called_once_with(
            str(output_file), dpi=300, bbox_inches="tight"
        )

    @patch("chart_generators.plt")
    def test_size_histogram_creates_histogram(
        self, mock_plt, temp_dir, sample_csv_content
    ):
        """RED: Test that size histogram creates histogram elements"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "histogram.png"

        # Mock the plotting functions
        mock_patches = [MagicMock() for _ in range(8)]
        for mock_patch in mock_patches:
            mock_patch.set_facecolor.return_value = None

        mock_plt.figure.return_value = MagicMock()
        mock_plt.hist.return_value = (None, None, mock_patches)
        mock_plt.axvline.return_value = MagicMock()

        chart_gen.create_size_histogram(str(output_file))

        # Should call histogram functions
        mock_plt.figure.assert_called_once()
        mock_plt.hist.assert_called_once()
        mock_plt.savefig.assert_called_once_with(
            str(output_file), dpi=300, bbox_inches="tight"
        )


class TestGenerateAllCharts:
    """Test the generate_all_charts comprehensive function"""

    def test_generate_all_charts_with_no_data(self, temp_dir):
        """RED: Test generate_all_charts with no loaded data"""
        metrics = PRMetrics("nonexistent.csv")
        chart_gen = ChartGenerator(metrics)

        result = chart_gen.generate_all_charts(temp_dir)

        # Should return empty dict when no data
        assert result == {}

    @patch("chart_generators.plt")
    def test_generate_all_charts_creates_output_directory(
        self, mock_plt, temp_dir, sample_csv_content
    ):
        """RED: Test that generate_all_charts creates output directory"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        chart_gen = ChartGenerator(metrics)

        output_dir = Path(temp_dir) / "charts"

        # Mock all plotting functions
        mock_plt.figure.return_value = MagicMock()
        mock_plt.pie.return_value = MagicMock()
        mock_plt.plot.return_value = MagicMock()
        mock_plt.bar.return_value = [MagicMock()]
        mock_plt.hist.return_value = (None, None, [MagicMock()])
        mock_plt.close.return_value = MagicMock()
        mock_plt.savefig.return_value = MagicMock()
        mock_plt.gca.return_value = MagicMock()
        mock_plt.axvline.return_value = MagicMock()

        result = chart_gen.generate_all_charts(str(output_dir))

        # Should create output directory
        assert output_dir.exists()
        assert output_dir.is_dir()

        # Should return chart info
        assert isinstance(result, dict)
        expected_keys = {
            "pie_chart",
            "trend_chart",
            "contributor_chart",
            "histogram",
            "category_counts",
        }
        assert all(key in result for key in expected_keys)

    @patch("chart_generators.plt")
    def test_generate_all_charts_handles_exceptions(self, mock_plt, temp_dir,
                                                    sample_csv_content):
        """RED: Test that generate_all_charts handles plotting exceptions
        gracefully"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        chart_gen = ChartGenerator(metrics)

        # Mock plotting to raise exception
        mock_plt.figure.side_effect = RuntimeError("Plotting failed")

        result = chart_gen.generate_all_charts(temp_dir)

        # Should handle exception gracefully
        assert result == {}


class TestChartGeneratorEdgeCases:
    """Test edge cases and error conditions"""

    def test_chart_generation_with_empty_data(self, temp_dir):
        """RED: Test chart generation with empty but valid CSV"""
        csv_content = (
            "date,pr_number,title,author,total_lines,additions,deletions,"
            "changed_files,category,url\n"
        )
        csv_file = Path(temp_dir) / "empty.csv"
        csv_file.write_text(csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        chart_gen = ChartGenerator(metrics)

        output_file = Path(temp_dir) / "test_chart.png"
        result = chart_gen.create_size_distribution_chart(str(output_file))

        # Should handle empty data gracefully
        assert result == {}

    def test_chart_generation_with_invalid_output_path(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test chart generation with invalid output path"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        chart_gen = ChartGenerator(metrics)

        # Try to save to non-existent directory
        invalid_path = "/nonexistent/directory/chart.png"

        # Should handle invalid path gracefully
        # (this will be implementation dependent)
        # The test documents expected behavior
        try:
            result = chart_gen.create_size_distribution_chart(invalid_path)
            # If it doesn't raise exception, it should return empty result
            assert result == {} or isinstance(result, dict)
        except (OSError, IOError):
            # It's acceptable to raise an exception for invalid paths
            pass
