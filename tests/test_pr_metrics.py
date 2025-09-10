#!/usr/bin/env python3
"""
TDD Tests for PR Metrics Module
Following Red-Green-Refactor cycle and FIRST principles
"""

from pathlib import Path

# Import the module under test
from pr_metrics import PRMetrics


class TestPRMetricsInstantiation:
    """Test PRMetrics class instantiation - Basic TDD cycle"""

    def test_can_create_pr_metrics_instance(self):
        """RED: Test that we can create a PRMetrics instance"""
        csv_file = "test.csv"
        metrics = PRMetrics(csv_file)
        assert metrics.csv_file == csv_file
        assert metrics.df is None

    def test_pr_metrics_initializes_with_empty_dataframe(self):
        """RED: Test initial state is correct"""
        metrics = PRMetrics("test.csv")
        assert not metrics.is_loaded


class TestPRMetricsDataLoading:
    """Test data loading functionality - TDD approach"""

    def test_load_data_returns_false_for_missing_file(self):
        """RED: Test that loading non-existent file returns False"""
        metrics = PRMetrics("nonexistent.csv")
        result = metrics.load_data()
        assert result is False
        assert not metrics.is_loaded

    def test_load_data_returns_true_for_valid_csv(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test that loading valid CSV returns True"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        result = metrics.load_data()

        assert result is True
        assert metrics.is_loaded
        assert len(metrics.df) == 3

    def test_load_data_handles_empty_csv(self, temp_dir):
        """RED: Test that empty CSV is handled gracefully"""
        csv_file = Path(temp_dir) / "empty.csv"
        csv_file.write_text("")

        metrics = PRMetrics(str(csv_file))
        result = metrics.load_data()

        assert result is False
        assert not metrics.is_loaded

    def test_load_data_handles_invalid_csv(self, temp_dir):
        """RED: Test that invalid CSV is handled gracefully"""
        csv_file = Path(temp_dir) / "invalid.csv"
        csv_file.write_text("invalid,csv,content\nwith,wrong,format")

        metrics = PRMetrics(str(csv_file))
        result = metrics.load_data()

        # Should handle gracefully - either load what it can or fail safely
        assert isinstance(result, bool)


class TestPRMetricsCategorization:
    """Test PR size categorization logic"""

    def test_categorize_pr_size_ideal(self):
        """RED: Test ideal size categorization"""
        metrics = PRMetrics("test.csv")
        result = metrics.categorize_pr_size(450)
        assert result == "Ideal (≤500)"

    def test_categorize_pr_size_good(self):
        """RED: Test good size categorization"""
        metrics = PRMetrics("test.csv")
        result = metrics.categorize_pr_size(800)
        assert result == "Good (501-1500)"

    def test_categorize_pr_size_large(self):
        """RED: Test large size categorization"""
        metrics = PRMetrics("test.csv")
        result = metrics.categorize_pr_size(1800)
        assert result == "Large (1501-2000)"

    def test_categorize_pr_size_too_large(self):
        """RED: Test too large categorization"""
        metrics = PRMetrics("test.csv")
        result = metrics.categorize_pr_size(2500)
        assert result == "Too Large (>2000)"

    def test_categorize_pr_size_boundary_values(self):
        """RED: Test boundary values"""
        metrics = PRMetrics("test.csv")

        # Test exact boundaries
        assert metrics.categorize_pr_size(500) == "Ideal (≤500)"
        assert metrics.categorize_pr_size(501) == "Good (501-1500)"
        assert metrics.categorize_pr_size(1500) == "Good (501-1500)"
        assert metrics.categorize_pr_size(1501) == "Large (1501-2000)"
        assert metrics.categorize_pr_size(2000) == "Large (1501-2000)"
        assert metrics.categorize_pr_size(2001) == "Too Large (>2000)"


class TestPRMetricsStatistics:
    """Test statistical calculations"""

    def test_get_size_distribution_with_no_data(self):
        """RED: Test distribution with no loaded data"""
        metrics = PRMetrics("test.csv")
        ideal, good, large, too_large = metrics.get_size_distribution()
        assert ideal == 0
        assert good == 0
        assert large == 0
        assert too_large == 0

    def test_get_averages_with_no_data(self):
        """RED: Test averages with no loaded data"""
        metrics = PRMetrics("test.csv")
        avg_lines, avg_files = metrics.get_averages()
        assert avg_lines == 0.0
        assert avg_files == 0.0

    def test_get_recent_trends_with_no_data(self):
        """RED: Test recent trends with no loaded data"""
        metrics = PRMetrics("test.csv")
        recent_lines, recent_files = metrics.get_recent_trends()
        assert recent_lines == 0.0
        assert recent_files == 0.0

    def test_get_statistics_summary_with_no_data(self):
        """RED: Test complete summary with no data"""
        metrics = PRMetrics("test.csv")
        stats = metrics.get_statistics_summary()

        expected_keys = {
            "total_prs",
            "avg_lines",
            "avg_files",
            "ideal_count",
            "good_count",
            "large_count",
            "too_large_count",
            "recent_avg_lines",
            "recent_avg_files",
        }

        assert set(stats.keys()) == expected_keys
        assert stats["total_prs"] == 0
        assert all(
            stats[key] == 0.0
            for key in [
                "avg_lines",
                "avg_files",
                "recent_avg_lines",
                "recent_avg_files",
            ]
        )


class TestPRMetricsWithLoadedData:
    """Test functionality with loaded data - Integration style"""

    def test_statistics_with_loaded_data(self, temp_dir, sample_csv_content):
        """RED: Test statistics calculations with real data"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()

        # Test size distribution
        ideal, good, large, too_large = metrics.get_size_distribution()
        assert ideal == 1  # 450 lines
        assert good == 1  # 850 lines
        assert large == 0
        assert too_large == 1  # 2200 lines

        # Test averages
        avg_lines, avg_files = metrics.get_averages()
        expected_avg_lines = (450 + 850 + 2200) / 3
        expected_avg_files = (8 + 12 + 45) / 3

        assert abs(avg_lines - expected_avg_lines) < 0.1
        assert abs(avg_files - expected_avg_files) < 0.1

    def test_top_contributors(self, temp_dir, sample_csv_content):
        """RED: Test top contributors functionality"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()

        contributors = metrics.get_top_contributors(5)

        # bob has 2 PRs, alice has 1
        assert contributors.iloc[0] == 2  # bob's count
        assert contributors.index[0] == "bob"
        assert contributors.iloc[1] == 1  # alice's count
        assert contributors.index[1] == "alice"

    def test_large_prs_filtering(self, temp_dir, sample_csv_content):
        """RED: Test large PRs filtering"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()

        large_prs = metrics.get_large_prs(min_lines=1500, count=10)

        # Only the 2200 line PR should be returned
        assert len(large_prs) == 1
        assert large_prs.iloc[0]["total_lines"] == 2200
        assert large_prs.iloc[0]["pr_number"] == 106


class TestPRMetricsEdgeCases:
    """Test edge cases and error conditions"""

    def test_get_recent_trends_with_few_prs(self, temp_dir):
        """RED: Test recent trends with fewer than 30 PRs"""
        csv_content = (
            "date,pr_number,title,author,total_lines,additions,deletions,"
            "changed_files,category,url\n"
            "2024-01-15T10:30:00Z,101,Small PR,alice,100,80,20,2,ideal,"
            "https://github.com/example/repo/pull/101"
        )

        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()

        recent_lines, recent_files = metrics.get_recent_trends(30)

        # Should use all available data when less than requested count
        assert recent_lines == 100.0
        assert recent_files == 2.0

    def test_empty_contributors_list(self, temp_dir):
        """RED: Test getting contributors from empty dataset"""
        csv_file = Path(temp_dir) / "empty.csv"
        csv_file.write_text(
            "date,pr_number,title,author,total_lines,additions,deletions,"
            "changed_files,category,url\n"
        )

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()

        contributors = metrics.get_top_contributors(10)
        assert len(contributors) == 0
