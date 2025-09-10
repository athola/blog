#!/usr/bin/env python3
"""Report Generation Module.

This module handles generating markdown reports from PR metrics data.
Separated from visualization and data processing for better maintainability.
"""

from datetime import datetime
from typing import Dict

import pandas as pd

from pr_metrics import PRMetrics


class ReportGenerator:
    """Generator for PR metrics markdown reports."""

    def __init__(self, metrics: PRMetrics):
        """Initialize with PRMetrics instance."""
        self.metrics = metrics

    def generate_summary_section(self, stats: Dict) -> str:
        """Generate the summary section of the report.

        Args:
            stats: Statistics dictionary from PRMetrics

        Returns:
            str: Markdown summary section
        """
        return f"""# PR Size Trend Report

Generated on: {datetime.now().strftime("%Y-%m-%d %H:%M:%S UTC")}

## Summary Statistics

- **Total PRs analyzed**: {stats['total_prs']}
- **Average lines changed per PR**: {stats['avg_lines']:.1f}
- **Average files changed per PR**: {stats['avg_files']:.1f}

## PR Size Distribution

"""

    def generate_distribution_table(self, stats: Dict) -> str:
        """Generate the size distribution table.

        Args:
            stats: Statistics dictionary from PRMetrics

        Returns:
            str: Markdown table for size distribution
        """
        total_prs = stats["total_prs"]
        if total_prs == 0:
            return "No data available for distribution analysis.\n"

        return f"""| Size Category | Count | Percentage |
|---------------|-------|------------|
| ✅ Ideal (≤500 lines) | {stats['ideal_count']} | {stats['ideal_count'] / total_prs * 100:.1f}% |
| 🟡 Good (501-1500 lines) | {stats['good_count']} | {stats['good_count'] / total_prs * 100:.1f}% |
| ⚠️ Large (1501-2000 lines) | {stats['large_count']} | {stats['large_count'] / total_prs * 100:.1f}% |
| ❌ Too Large (>2000 lines) | {stats['too_large_count']} | {stats['too_large_count'] / total_prs * 100:.1f}% |

## Recent Trends (Last 30 PRs)

- **Recent average lines changed per PR**: {stats['recent_avg_lines']:.1f}
- **Recent average files changed per PR**: {stats['recent_avg_files']:.1f}

"""

    def generate_contributors_section(self) -> str:
        """Generate the top contributors section.

        Returns:
            str: Markdown section for top contributors
        """
        if not self.metrics.is_loaded:
            return ""

        top_contributors = self.metrics.get_top_contributors(10)
        if top_contributors.empty:
            return ""

        report = "\n## Top Contributors (By PR Count)\n\n"
        report += "| Contributor | PR Count | Avg Size |\n"
        report += "|-------------|----------|----------|\n"

        for author, count in top_contributors.items():
            author_prs = self.metrics.df[self.metrics.df["author"] == author]
            avg_size = author_prs["total_lines"].mean()
            report += f"| {author} | {count} | {avg_size:.1f} |\n"

        return report

    def generate_large_prs_section(self) -> str:
        """Generate the large PRs section.

        Returns:
            str: Markdown section for recent large PRs
        """
        if not self.metrics.is_loaded:
            return ""

        large_prs = self.metrics.get_large_prs(min_lines=1500, count=10)
        if large_prs.empty:
            return ""

        report = "\n## Recent Large PRs (>1500 lines)\n\n"
        report += "| PR | Title | Author | Lines | Date |\n"
        report += "|----|-------|--------|-------|------|\n"

        for _, pr in large_prs.iterrows():
            # Truncate title if too long
            title = (
                pr["title"][:47] + "..."
                if len(pr["title"]) > 50
                else pr["title"]
            )
            date_str = (
                pr["date"].strftime("%Y-%m-%d")
                if pd.notnull(pr["date"])
                else "N/A"
            )
            pr_number = pr["pr_number"]
            url = pr["url"]
            author = pr["author"]
            total_lines = pr["total_lines"]
            report_line = f"| [#{pr_number}]({url}) | {title} | {author} "
            report_line += f"| {total_lines} | {date_str} |\n"
            report += report_line

        return report

    def generate_insights_section(self, stats: Dict) -> str:
        """Generate insights section based on metrics.

        Args:
            stats: Statistics dictionary from PRMetrics

        Returns:
            str: Markdown section with insights
        """
        if not self.metrics.is_loaded or stats["total_prs"] == 0:
            return ""

        insights = "\n## Key Insights\n\n"

        # Size distribution insights
        total_prs = stats["total_prs"]
        ideal_pct = (stats["ideal_count"] / total_prs) * 100
        too_large_pct = (stats["too_large_count"] / total_prs) * 100

        if ideal_pct >= 60:
            insights += "✅ **Excellent PR sizing**: Over 60% of PRs are in the ideal size range.\n\n"
        elif ideal_pct >= 40:
            insights += "🟡 **Good PR sizing**: 40-60% of PRs are in the ideal size range.\n\n"
        else:
            insights += "⚠️ **PR sizing needs improvement**: Less than 40% of PRs are in the ideal size range.\n\n"

        if too_large_pct > 10:
            insights += f"🚨 **Too many large PRs**: {too_large_pct:.1f}% of PRs exceed the 2000-line limit. Consider breaking these down.\n\n"
        elif too_large_pct > 5:
            insights += f"⚠️ **Some large PRs**: {too_large_pct:.1f}% of PRs exceed the 2000-line limit.\n\n"
        else:
            insights += "✅ **Good size control**: Very few PRs exceed the 2000-line limit.\n\n"

        # Recent trends
        recent_avg = stats["recent_avg_lines"]
        overall_avg = stats["avg_lines"]

        if recent_avg > overall_avg * 1.2:
            insights += f"📈 **Recent PRs are getting larger**: Recent average ({recent_avg:.0f} lines) is significantly higher than overall average ({overall_avg:.0f} lines).\n\n"
        elif recent_avg < overall_avg * 0.8:
            insights += f"📉 **Recent PRs are getting smaller**: Recent average ({recent_avg:.0f} lines) is lower than overall average ({overall_avg:.0f} lines).\n\n"
        else:
            insights += f"➡️ **Consistent PR sizing**: Recent average ({recent_avg:.0f} lines) is similar to overall average ({overall_avg:.0f} lines).\n\n"

        return insights

    def generate_recommendations_section(self, stats: Dict) -> str:
        """Generate recommendations based on metrics.

        Args:
            stats: Statistics dictionary from PRMetrics

        Returns:
            str: Markdown section with recommendations
        """
        if not self.metrics.is_loaded or stats["total_prs"] == 0:
            return ""

        recommendations = "\n## Recommendations\n\n"

        total_prs = stats["total_prs"]
        ideal_pct = (stats["ideal_count"] / total_prs) * 100
        too_large_pct = (stats["too_large_count"] / total_prs) * 100

        if ideal_pct < 50:
            recommendations += "1. **Focus on smaller PRs**: Aim to break down features into smaller, more focused changes.\n"
            recommendations += "2. **Use feature flags**: Consider using feature flags to ship incomplete features safely.\n"
            recommendations += "3. **Preparatory PRs**: Create separate PRs for infrastructure changes before feature implementation.\n\n"

        if too_large_pct > 5:
            recommendations += "1. **Review large PR process**: Establish a review process for PRs over 1500 lines.\n"
            recommendations += "2. **Breaking down strategies**: Document strategies for splitting large changes.\n"
            recommendations += "3. **Reviewer assignment**: Assign multiple reviewers for large PRs to ensure thorough review.\n\n"

        if stats["recent_avg_lines"] > 1000:
            recommendations += "1. **Recent trend concern**: Recent PRs are averaging over 1000 lines - consider smaller iterations.\n"
            recommendations += "2. **Sprint planning**: Break down larger features at the planning stage.\n\n"

        recommendations += "### General Best Practices\n\n"
        recommendations += "- **Single responsibility**: Each PR should address one specific concern\n"
        recommendations += "- **Incremental changes**: Build features incrementally with each PR adding value\n"
        recommendations += "- **Review guidelines**: Establish clear guidelines for PR review based on size\n"
        recommendations += "- **Automation**: Use automated checks to enforce size limits where appropriate\n\n"

        return recommendations

    def create_full_report(self) -> str:
        """Generate comprehensive statistics report as markdown.

        Returns:
            str: Complete markdown report
        """
        if not self.metrics.is_loaded:
            return f"""# PR Size Trend Report

Generated on: {datetime.now().strftime("%Y-%m-%d %H:%M:%S UTC")}

No data available."""

        # Get comprehensive statistics
        stats = self.metrics.get_statistics_summary()

        # Generate all report sections
        report = self.generate_summary_section(stats)
        report += self.generate_distribution_table(stats)
        report += self.generate_contributors_section()
        report += self.generate_large_prs_section()
        report += self.generate_insights_section(stats)
        report += self.generate_recommendations_section(stats)

        return report
