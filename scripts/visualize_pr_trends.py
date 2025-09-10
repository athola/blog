#!/usr/bin/env python3
"""PR Size Trend Visualization Script (Refactored).

This script serves as the main entry point for PR metrics visualization.
It uses modular components for better maintainability:
- pr_metrics.py: Data loading and processing
- chart_generators.py: Chart creation
- report_generator.py: Report generation
"""

import sys
import os

from pr_metrics import PRMetrics
from chart_generators import ChartGenerator
from report_generator import ReportGenerator


def main():
    """Run the main visualization process using modular components."""
    if len(sys.argv) != 2:
        print("Usage: python3 visualize_pr_trends.py <path_to_metrics_csv>")
        sys.exit(1)

    csv_file = sys.argv[1]

    # Create output directories
    output_dir = ".github/reports"
    viz_dir = os.path.join(output_dir, "visualizations")
    os.makedirs(output_dir, exist_ok=True)
    os.makedirs(viz_dir, exist_ok=True)

    # Load data using PRMetrics class
    metrics = PRMetrics(csv_file)
    if not metrics.load_data():
        print("No data available for visualization.")
        return

    try:
        # Generate visualizations using ChartGenerator
        chart_generator = ChartGenerator(metrics)
        chart_info = chart_generator.generate_all_charts(viz_dir)

        if not chart_info:
            print("Error generating charts.")
            return

        # Generate report using ReportGenerator
        report_generator = ReportGenerator(metrics)
        report_content = report_generator.create_full_report()

        # Save report
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
