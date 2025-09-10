#!/usr/bin/env python3
"""
TDD Tests for Report Generator Module
Following Red-Green-Refactor cycle and FIRST principles
"""

import pytest
import pandas as pd
from pathlib import Path
from datetime import datetime

# Import modules under test
from pr_metrics import PRMetrics
from report_generator import ReportGenerator


class TestReportGeneratorInstantiation:
    """Test ReportGenerator class instantiation - Basic TDD cycle"""

    def test_can_create_report_generator_instance(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test that we can create a ReportGenerator instance"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()

        report_gen = ReportGenerator(metrics)
        assert report_gen.metrics == metrics

    def test_report_generator_requires_metrics_object(self):
        """RED: Test that ReportGenerator requires PRMetrics instance"""
        metrics = PRMetrics("test.csv")
        report_gen = ReportGenerator(metrics)
        assert report_gen.metrics is not None


class TestReportGenerationWithoutData:
    """Test report generation behavior when no data is loaded"""

    def test_summary_section_with_no_data(self):
        """RED: Test summary section generation with no data"""
        metrics = PRMetrics("nonexistent.csv")
        report_gen = ReportGenerator(metrics)

        stats = {"total_prs": 0, "avg_lines": 0.0, "avg_files": 0.0}

        result = report_gen.generate_summary_section(stats)

        # Should contain basic structure
        assert "# PR Size Trend Report" in result
        assert "**Total PRs analyzed**: 0" in result
        assert "**Average lines changed per PR**: 0.0" in result

    def test_distribution_table_with_no_data(self):
        """RED: Test distribution table with no data"""
        metrics = PRMetrics("nonexistent.csv")
        report_gen = ReportGenerator(metrics)

        stats = {
            "total_prs": 0,
            "ideal_count": 0,
            "good_count": 0,
            "large_count": 0,
            "too_large_count": 0,
            "recent_avg_lines": 0.0,
            "recent_avg_files": 0.0,
        }

        result = report_gen.generate_distribution_table(stats)

        # Should handle division by zero gracefully
        assert "No data available" in result

    def test_contributors_section_with_no_data(self):
        """RED: Test contributors section with no data"""
        metrics = PRMetrics("nonexistent.csv")
        report_gen = ReportGenerator(metrics)

        result = report_gen.generate_contributors_section()

        # Should return empty string for no data
        assert result == ""

    def test_large_prs_section_with_no_data(self):
        """RED: Test large PRs section with no data"""
        metrics = PRMetrics("nonexistent.csv")
        report_gen = ReportGenerator(metrics)

        result = report_gen.generate_large_prs_section()

        # Should return empty string for no data
        assert result == ""

    def test_full_report_with_no_data(self):
        """RED: Test complete report generation with no data"""
        metrics = PRMetrics("nonexistent.csv")
        report_gen = ReportGenerator(metrics)

        result = report_gen.create_full_report()

        # Should contain basic structure
        assert "# PR Size Trend Report" in result
        assert "No data available" in result
        assert datetime.now().strftime("%Y-%m-%d") in result


class TestReportGenerationWithData:
    """Test report generation with loaded data"""

    def test_summary_section_with_real_data(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test summary section with actual data"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        stats = metrics.get_statistics_summary()
        result = report_gen.generate_summary_section(stats)

        # Should contain actual statistics
        assert "**Total PRs analyzed**: 3" in result
        assert "**Average lines changed per PR**:" in result
        assert float("1166.7") == pytest.approx(
            float(
                result.split("**Average lines changed per PR**: ")[1]
                .split("\n")[0]
            ),
            rel=1e-1,
        )

    def test_distribution_table_with_real_data(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test distribution table with actual data"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        stats = metrics.get_statistics_summary()
        result = report_gen.generate_distribution_table(stats)

        # Should contain actual percentages
        assert "| ✅ Ideal (≤500 lines) | 1 | 33.3% |" in result
        assert "| 🟡 Good (501-1500 lines) | 1 | 33.3% |" in result
        assert "| ❌ Too Large (>2000 lines) | 1 | 33.3% |" in result

    def test_contributors_section_with_real_data(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test contributors section with actual data"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        result = report_gen.generate_contributors_section()

        # Should contain contributor information
        assert "## Top Contributors (By PR Count)" in result
        assert "| bob | 2 |" in result  # bob has 2 PRs
        assert "| alice | 1 |" in result  # alice has 1 PR

    def test_large_prs_section_with_real_data(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test large PRs section with actual data"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        result = report_gen.generate_large_prs_section()

        # Should contain large PR information
        assert "## Recent Large PRs (>1500 lines)" in result
        assert "[#106]" in result  # PR 106 is > 1500 lines
        assert "2200" in result  # 2200 lines

    def test_insights_section_with_real_data(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test insights generation with actual data"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        stats = metrics.get_statistics_summary()
        result = report_gen.generate_insights_section(stats)

        # Should contain insights
        assert "## Key Insights" in result
        # With 33.3% ideal, should suggest improvement
        assert (
            "PR sizing needs improvement" in result
            or "Good PR sizing" in result
        )

    def test_recommendations_section_with_real_data(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test recommendations generation with actual data"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        stats = metrics.get_statistics_summary()
        result = report_gen.generate_recommendations_section(stats)

        # Should contain recommendations
        assert "## Recommendations" in result
        assert "General Best Practices" in result


class TestReportFormatting:
    """Test report formatting and structure"""

    def test_report_has_proper_markdown_structure(
        self, temp_dir, sample_csv_content
    ):
        """RED: Test that generated report has proper markdown structure"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        result = report_gen.create_full_report()

        # Should have proper markdown headers
        assert result.startswith("# PR Size Trend Report")
        assert "## Summary Statistics" in result
        assert "## PR Size Distribution" in result

        # Should have proper table formatting
        assert "|-------|" in result  # Table separator
        assert (
            "| ✅" in result or "| 🟡" in result
        )  # Table content with emojis

    def test_report_contains_timestamp(self, temp_dir, sample_csv_content):
        """RED: Test that report contains generation timestamp"""
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(sample_csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        result = report_gen.create_full_report()

        # Should contain current date
        current_date = datetime.now().strftime("%Y-%m-%d")
        assert current_date in result
        assert "Generated on:" in result

    def test_long_title_truncation(self, temp_dir):
        """RED: Test that long PR titles are properly truncated"""
        csv_content = (
            "date,pr_number,title,author,total_lines,additions,deletions,"
            "changed_files,category,url\n"
            "2024-01-20T13:20:00Z,106,This is a very long PR title that "
            "should be truncated because it exceeds the reasonable length "
            "for display in a table format,bob,2200,2000,200,45,too-large,"
            "https://github.com/example/repo/pull/106"
        )

        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        result = report_gen.generate_large_prs_section()

        # Should truncate long titles
        assert "..." in result
        assert (
            len([line for line in result.split("\n") if "[#106]" in line][0])
            < 200
        )


class TestReportEdgeCases:
    """Test edge cases and error conditions"""

    def test_report_with_missing_date_fields(self, temp_dir):
        """RED: Test report generation with missing date fields"""
        csv_content = (
            "date,pr_number,title,author,total_lines,additions,deletions,"
            "changed_files,category,url\n"
            ",101,Missing date PR,alice,450,380,70,8,ideal,"
            "https://github.com/example/repo/pull/101"
        )

        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text(csv_content)

        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        result = report_gen.generate_large_prs_section()

        # Should handle missing dates gracefully
        assert "N/A" in result or result == ""

    def test_insights_with_edge_case_percentages(self, temp_dir):
        """RED: Test insights generation with edge case percentages"""
        # Create a valid CSV file with minimal data
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text("date,pr_number,title,author,total_lines,additions,deletions,changed_files,category,url\n2024-01-01,1,Test,user,100,100,0,1,ideal,http://example.com")
        
        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        # Test with exactly 60% ideal
        stats = {
            "total_prs": 10,
            "ideal_count": 6,
            "good_count": 2,
            "large_count": 1,
            "too_large_count": 1,
            "recent_avg_lines": 500.0,
            "recent_avg_files": 5.0,
            "avg_lines": 600.0,
            "avg_files": 6.0,
        }

        result = report_gen.generate_insights_section(stats)

        # Should recognize 60% as excellent
        assert "Excellent PR sizing" in result

    def test_recommendations_with_various_scenarios(self, temp_dir):
        """RED: Test recommendations for different data scenarios"""
        # Create a valid CSV file with minimal data
        csv_file = Path(temp_dir) / "test.csv"
        csv_file.write_text("date,pr_number,title,author,total_lines,additions,deletions,changed_files,category,url\n2024-01-01,1,Test,user,100,100,0,1,ideal,http://example.com")
        
        metrics = PRMetrics(str(csv_file))
        metrics.load_data()
        report_gen = ReportGenerator(metrics)

        # Scenario: High percentage of too-large PRs
        stats = {
            "total_prs": 10,
            "ideal_count": 2,
            "good_count": 2,
            "large_count": 1,
            "too_large_count": 5,  # 50% too large
            "recent_avg_lines": 1500.0,  # High recent average
            "recent_avg_files": 15.0,
            "avg_lines": 1200.0,
            "avg_files": 12.0,
        }

        result = report_gen.generate_recommendations_section(stats)

        # Should recommend breaking down large PRs and recent trend concerns
        assert "Review large PR process" in result
        assert "Recent trend concern" in result
