#!/usr/bin/env python3
"""PR Size Trend Visualization Script.

This script generates visualizations from PR size metrics data.
The visualizations are treated as artifacts and not committed to the repository.
"""

import sys
import os
from datetime import datetime

import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.dates as mdates
import seaborn as sns
import numpy as np

# Set style for better-looking plots
plt.style.use("seaborn-v0_8")
sns.set_palette("husl")


def load_metrics_data(csv_file):
    """Load PR size metrics from CSV file."""
    if not os.path.exists(csv_file):
        print(f"Metrics file {csv_file} not found.")
        return None

    try:
        df = pd.read_csv(csv_file)
        # Convert date column to datetime
        df["date"] = pd.to_datetime(df["date"])
        return df
    except (pd.errors.EmptyDataError, pd.errors.ParserError, ValueError) as e:
        print(f"Error loading metrics data: {e}")
        return None


def create_size_distribution_chart(df, output_file):
    """Create a pie chart showing PR size distribution."""

    # Define size categories
    def categorize_pr_size(total_lines):
        if total_lines <= 500:
            return "Ideal (≤500)"
        if total_lines <= 1500:  # Changed from elif to if
            return "Good (501-1500)"
        if total_lines <= 2000:  # Changed from elif to if
            return "Large (1501-2000)"
        return "Too Large (>2000)"  # Changed from else to return

    df["size_category"] = df["total_lines"].apply(categorize_pr_size)

    # Count categories
    category_counts = df["size_category"].value_counts()

    # Create pie chart
    plt.figure(figsize=(10, 8))
    colors = [
        "#28a745",
        "#ffc107",
        "#fd7e14",
        "#dc3545",
    ]  # Green, Yellow, Orange, Red

    plt.pie(
        category_counts.values,
        labels=category_counts.index,
        autopct="%1.1f%%",
        colors=colors,
        startangle=90,
        explode=(0.05, 0.05, 0.05, 0.05),
    )
    plt.title("PR Size Distribution", fontsize=16, pad=20)

    plt.tight_layout()
    plt.savefig(output_file, dpi=300, bbox_inches="tight")
    plt.close()

    return category_counts


def create_trend_line_chart(df, output_file):
    """Create a line chart showing PR size trends over time."""
    # Group by date and calculate average size
    daily_avg = (
        df.groupby(df["date"].dt.date)["total_lines"].mean().reset_index()
    )
    daily_avg["date"] = pd.to_datetime(daily_avg["date"])

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


def create_contributor_chart(df, output_file):
    """Create a bar chart showing top contributors by PR count."""
    # Get top 10 contributors
    top_contributors = df["author"].value_counts().head(10)

    plt.figure(figsize=(12, 6))
    bars = plt.bar(
        range(len(top_contributors)),
        top_contributors.values,
        color=plt.get_cmap("tab10")(np.linspace(0, 1, len(top_contributors))),
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


def calculate_size_distribution(df):
    """Calculate PR size distribution counts."""
    ideal_count = len(df[df["total_lines"] <= 500])
    good_count = len(
        df[(df["total_lines"] > 500) & (df["total_lines"] <= 1500)]
    )
    large_count = len(
        df[(df["total_lines"] > 1500) & (df["total_lines"] <= 2000)]
    )
    too_large_count = len(df[df["total_lines"] > 2000])
    return ideal_count, good_count, large_count, too_large_count


def calculate_averages(df):
    """Calculate average metrics."""
    avg_lines = df["total_lines"].mean()
    avg_files = df["changed_files"].mean()
    return avg_lines, avg_files


def get_recent_trends(df):
    """Get recent trends from last 30 PRs."""
    recent_df = df.tail(30) if len(df) >= 30 else df
    recent_avg_lines = (
        recent_df["total_lines"].mean() if not recent_df.empty else 0
    )
    recent_avg_files = (
        recent_df["changed_files"].mean() if not recent_df.empty else 0
    )
    return recent_avg_lines, recent_avg_files


def generate_summary_section(total_prs, avg_lines, avg_files):
    """Generate the summary section of the report."""
    return f"""# PR Size Trend Report

Generated on: {datetime.now().strftime("%Y-%m-%d %H:%M:%S UTC")}

## Summary Statistics

- **Total PRs analyzed**: {total_prs}
- **Average lines changed per PR**: {avg_lines:.1f}
- **Average files changed per PR**: {avg_files:.1f}

## PR Size Distribution

"""


def generate_distribution_table(stats):
    """Generate the size distribution table."""
    return f"""| Size Category | Count | Percentage |
|---------------|-------|------------|
| ✅ Ideal (≤500 lines) | {stats['ideal_count']} |
{stats['ideal_count'] / stats['total_prs'] * 100:.1f}% |
| 🟡 Good (501-1500 lines) | {stats['good_count']} |
{stats['good_count'] / stats['total_prs'] * 100:.1f}% |
| ⚠️ Large (1501-2000 lines) | {stats['large_count']} |
{stats['large_count'] / stats['total_prs'] * 100:.1f}% |
| ❌ Too Large (>2000 lines) | {stats['too_large_count']} |
{stats['too_large_count'] / stats['total_prs'] * 100:.1f}% |

## Recent Trends (Last 30 PRs)

- **Recent average lines changed per PR**: {stats['recent_avg_lines']:.1f}
- **Recent average files changed per PR**: {stats['recent_avg_files']:.1f}
"""


def generate_contributors_section(df):
    """Generate the top contributors section."""
    top_contributors = df["author"].value_counts().head(10)
    if top_contributors.empty:
        return ""

    report = "\n## Top Contributors (By PR Count)\n\n"
    report += "| Contributor | PR Count | Avg Size |\n"
    report += "|-------------|----------|----------|\n"

    for author, count in top_contributors.items():
        author_prs = df[df["author"] == author]
        avg_size = author_prs["total_lines"].mean()
        report += f"| {author} | {count} | {avg_size:.1f} |\n"

    return report


def generate_large_prs_section(df):
    """Generate the large PRs section."""
    large_prs = df[df["total_lines"] > 1500].tail(10)
    if large_prs.empty:
        return ""

    report = "\n## Recent Large PRs (>1500 lines)\n\n"
    report += "| PR | Title | Author | Lines | Date |\n"
    report += "|----|-------|--------|-------|------|\n"

    for _, pr in large_prs.iterrows():
        # Truncate title if too long
        title = (
            pr["title"][:47] + "..." if len(pr["title"]) > 50 else pr["title"]
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


def create_statistics_report(df):
    """Generate statistics report as markdown."""
    total_prs = len(df)

    # Early return if no data
    if total_prs == 0:
        return f"""# PR Size Trend Report

Generated on: {datetime.now().strftime("%Y-%m-%d %H:%M:%S UTC")}

No data available."""

    # Calculate metrics using helper functions
    ideal_count, good_count, large_count, too_large_count = (
        calculate_size_distribution(df)
    )
    avg_lines, avg_files = calculate_averages(df)
    recent_avg_lines, recent_avg_files = get_recent_trends(df)

    # Generate report sections
    report = generate_summary_section(total_prs, avg_lines, avg_files)
    stats = {
        "total_prs": total_prs,
        "ideal_count": ideal_count,
        "good_count": good_count,
        "large_count": large_count,
        "too_large_count": too_large_count,
        "recent_avg_lines": recent_avg_lines,
        "recent_avg_files": recent_avg_files,
    }
    report += generate_distribution_table(stats)
    report += generate_contributors_section(df)
    report += generate_large_prs_section(df)

    return report


def main():
    """Run the main visualization process."""
    if len(sys.argv) != 2:
        print("Usage: python3 visualize_pr_trends.py <path_to_metrics_csv>")
        sys.exit(1)

    csv_file = sys.argv[1]

    # Create output directories
    output_dir = ".github/reports"
    viz_dir = os.path.join(output_dir, "visualizations")
    os.makedirs(viz_dir, exist_ok=True)

    # Load data
    df = load_metrics_data(csv_file)
    if df is None or df.empty:
        print("No data available for visualization.")
        return

    print(f"Loaded {len(df)} PR records for visualization.")

    # Generate visualizations (treated as artifacts)
    try:
        # Size distribution pie chart
        pie_chart_file = os.path.join(viz_dir, "pr_size_distribution.png")
        create_size_distribution_chart(df, pie_chart_file)
        print(f"Created size distribution chart: {pie_chart_file}")

        # Trend line chart
        trend_chart_file = os.path.join(viz_dir, "pr_size_trend.png")
        create_trend_line_chart(df, trend_chart_file)
        print(f"Created trend line chart: {trend_chart_file}")

        # Contributor chart
        contributor_chart_file = os.path.join(viz_dir, "top_contributors.png")
        create_contributor_chart(df, contributor_chart_file)
        print(f"Created contributor chart: {contributor_chart_file}")

        # Generate statistics report
        report_content = create_statistics_report(df)
        report_file = os.path.join(output_dir, "pr_size_trend.md")
        with open(report_file, "w", encoding="utf-8") as f:
            f.write(report_content)
        print(f"Created statistics report: {report_file}")

        print("\nVisualization generation complete!")
        print(f"Visualizations saved as artifacts to: {viz_dir}")
        print(f"Report saved to: {report_file}")
        print(
            "Note: Visualization PNG files are treated as artifacts "
            "and not committed to the repository."
        )

    except (ImportError, RuntimeError, ValueError) as e:
        print(f"Error generating visualizations: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
