#!/usr/bin/env python3
"""Chart Generation Module.

This module handles all visualization chart generation logic.
Separated from data processing for better maintainability.
"""

import os
from typing import Dict

import matplotlib

# Configure matplotlib for headless environments (GitHub Actions)
matplotlib.use("Agg")  # Use non-GUI backend
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
import seaborn as sns
import numpy as np

from pr_metrics import PRMetrics

# Set style for better-looking plots with fallback
try:
    plt.style.use("seaborn-v0_8")
except OSError:
    # Fallback for older seaborn versions
    try:
        plt.style.use("seaborn")
    except OSError:
        # Use matplotlib default if seaborn styles not available
        plt.style.use("default")
        print(
            "Warning: Using default matplotlib style (seaborn not available)"
        )

try:
    sns.set_palette("husl")
except Exception:
    print("Warning: Could not set seaborn palette, using default")


class ChartGenerator:
    """Generator for PR metrics visualization charts."""

    def __init__(self, metrics: PRMetrics):
        """Initialize with PRMetrics instance."""
        self.metrics = metrics

    def create_size_distribution_chart(self, output_file: str) -> Dict:
        """Create a pie chart showing PR size distribution.

        Args:
            output_file: Path where chart will be saved

        Returns:
            Dict: Category counts for use in reports
        """
        if not self.metrics.is_loaded:
            return {}

        # Add size category column
        self.metrics.df["size_category"] = self.metrics.df[
            "total_lines"
        ].apply(self.metrics.categorize_pr_size)

        # Count categories
        category_counts = self.metrics.df["size_category"].value_counts()

        # Create pie chart
        plt.figure(figsize=(10, 8))
        colors = [
            "#28a745",
            "#ffc107",
            "#fd7e14",
            "#dc3545",
        ]  # Green, Yellow, Orange, Red

        # Create explode tuple matching the number of categories
        explode_tuple = tuple(0.05 for _ in range(len(category_counts)))

        plt.pie(
            category_counts.values,
            labels=category_counts.index,
            autopct="%1.1f%%",
            colors=colors[
                : len(category_counts)
            ],  # Match colors to categories
            startangle=90,
            explode=explode_tuple,
        )
        plt.title("PR Size Distribution", fontsize=16, pad=20)

        plt.tight_layout()
        plt.savefig(output_file, dpi=300, bbox_inches="tight")
        plt.close()

        return dict(category_counts)

    def create_trend_line_chart(self, output_file: str) -> None:
        """Create a line chart showing PR size trends over time.

        Args:
            output_file: Path where chart will be saved
        """
        if not self.metrics.is_loaded:
            return

        daily_avg = self.metrics.get_daily_averages()
        if daily_avg.empty:
            return

        plt.figure(figsize=(12, 6))
        plt.plot(
            daily_avg["date"],
            daily_avg["total_lines"],
            marker="o",
            linewidth=2,
            markersize=4,
        )

        plt.xlabel("Date")
        plt.ylabel("Average PR Size (lines)")
        plt.title("PR Size Trends Over Time")
        plt.grid(True, alpha=0.3)

        # Format x-axis dates
        plt.gca().xaxis.set_major_formatter(mdates.DateFormatter("%Y-%m-%d"))
        plt.gca().xaxis.set_major_locator(mdates.MonthLocator())
        plt.xticks(rotation=45)

        plt.tight_layout()
        plt.savefig(output_file, dpi=300, bbox_inches="tight")
        plt.close()

    def create_contributor_chart(self, output_file: str) -> None:
        """Create a bar chart showing top contributors by PR count.

        Args:
            output_file: Path where chart will be saved
        """
        if not self.metrics.is_loaded:
            return

        # Get top 10 contributors
        top_contributors = self.metrics.get_top_contributors(10)
        if top_contributors.empty:
            return

        plt.figure(figsize=(12, 6))
        bars = plt.bar(
            range(len(top_contributors)),
            top_contributors.values,
            color=plt.get_cmap("tab10")(
                np.linspace(0, 1, len(top_contributors))
            ),
        )

        plt.xlabel("Contributors")
        plt.ylabel("Number of PRs")
        plt.title("Top Contributors by PR Count")
        plt.xticks(
            range(len(top_contributors)),
            top_contributors.index,
            rotation=45,
            ha="right",
        )

        # Add value labels on bars
        for bar_obj in bars:
            height = bar_obj.get_height()
            plt.text(
                bar_obj.get_x() + bar_obj.get_width() / 2.0,
                height,
                f"{int(height)}",
                ha="center",
                va="bottom",
            )

        plt.tight_layout()
        plt.savefig(output_file, dpi=300, bbox_inches="tight")
        plt.close()

    def create_size_histogram(self, output_file: str) -> None:
        """Create a histogram showing distribution of PR sizes.

        Args:
            output_file: Path where chart will be saved
        """
        if not self.metrics.is_loaded:
            return

        plt.figure(figsize=(12, 6))

        # Create histogram with custom bins
        bins = [0, 500, 1000, 1500, 2000, 2500, 3000, 5000, 10000]
        colors = [
            "#28a745",
            "#90ee90",
            "#ffc107",
            "#fd7e14",
            "#dc3545",
            "#8b0000",
            "#4b0000",
            "#2b0000",
        ]

        n, bins, patches = plt.hist(
            self.metrics.df["total_lines"],
            bins=bins,
            edgecolor="black",
            alpha=0.7,
        )

        # Color each bar based on PR size category
        for i, patch in enumerate(patches):
            patch.set_facecolor(colors[i % len(colors)])

        plt.xlabel("PR Size (lines changed)")
        plt.ylabel("Number of PRs")
        plt.title("Distribution of PR Sizes")
        plt.grid(True, alpha=0.3)

        # Add vertical lines for size thresholds
        plt.axvline(
            x=500,
            color="green",
            linestyle="--",
            alpha=0.7,
            label="Ideal limit",
        )
        plt.axvline(
            x=1500,
            color="orange",
            linestyle="--",
            alpha=0.7,
            label="Good limit",
        )
        plt.axvline(
            x=2000, color="red", linestyle="--", alpha=0.7, label="Max limit"
        )
        plt.legend()

        plt.tight_layout()
        plt.savefig(output_file, dpi=300, bbox_inches="tight")
        plt.close()

    def generate_all_charts(self, output_dir: str) -> Dict:
        """Generate all visualization charts.

        Args:
            output_dir: Directory where charts will be saved

        Returns:
            Dict: Information about generated charts
        """
        if not self.metrics.is_loaded:
            return {}

        os.makedirs(output_dir, exist_ok=True)
        chart_info = {}

        try:
            # Size distribution pie chart
            pie_chart_file = os.path.join(
                output_dir, "pr_size_distribution.png"
            )
            category_counts = self.create_size_distribution_chart(
                pie_chart_file
            )
            chart_info["pie_chart"] = pie_chart_file
            chart_info["category_counts"] = category_counts
            print(f"Created size distribution chart: {pie_chart_file}")

            # Trend line chart
            trend_chart_file = os.path.join(output_dir, "pr_size_trend.png")
            self.create_trend_line_chart(trend_chart_file)
            chart_info["trend_chart"] = trend_chart_file
            print(f"Created trend line chart: {trend_chart_file}")

            # Contributor chart
            contributor_chart_file = os.path.join(
                output_dir, "top_contributors.png"
            )
            self.create_contributor_chart(contributor_chart_file)
            chart_info["contributor_chart"] = contributor_chart_file
            print(f"Created contributor chart: {contributor_chart_file}")

            # Size histogram
            histogram_file = os.path.join(output_dir, "pr_size_histogram.png")
            self.create_size_histogram(histogram_file)
            chart_info["histogram"] = histogram_file
            print(f"Created size histogram: {histogram_file}")

            return chart_info

        except (ImportError, RuntimeError, ValueError) as e:
            print(f"Error generating charts: {e}")
            return {}
