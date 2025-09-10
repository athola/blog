#!/usr/bin/env python3
"""PR Metrics Module.

This module handles loading and processing PR size metrics data.
Separated from visualization logic for better maintainability.
"""

import os
from datetime import datetime
from typing import Dict, List, Optional, Tuple

import pandas as pd


class PRMetrics:
    """Handler for PR size metrics data."""

    def __init__(self, csv_file: str):
        """Initialize with metrics CSV file path."""
        self.csv_file = csv_file
        self.df = None

    def load_data(self) -> bool:
        """Load PR size metrics from CSV file.

        Returns:
            bool: True if data loaded successfully, False otherwise
        """
        if not os.path.exists(self.csv_file):
            print(f"Metrics file {self.csv_file} not found.")
            return False

        try:
            self.df = pd.read_csv(self.csv_file)

            # Check for required columns
            required_columns = ["date", "total_lines", "pr_number"]
            missing_columns = [
                col for col in required_columns if col not in self.df.columns
            ]

            if missing_columns:
                print(f"Error: Missing required columns: {missing_columns}")
                self.df = None
                return False

            # Convert date column to datetime
            self.df["date"] = pd.to_datetime(self.df["date"])
            print(f"Loaded {len(self.df)} PR records from {self.csv_file}")
            return True
        except (
            pd.errors.EmptyDataError,
            pd.errors.ParserError,
            ValueError,
            KeyError,
        ) as e:
            print(f"Error loading metrics data: {e}")
            self.df = None
            return False

    @property
    def is_loaded(self) -> bool:
        """Check if data is loaded and not empty."""
        return self.df is not None and not self.df.empty

    def categorize_pr_size(self, total_lines: int) -> str:
        """Categorize PR size based on line count.

        Args:
            total_lines: Total lines changed in PR

        Returns:
            str: Size category string
        """
        if total_lines <= 500:
            return "Ideal (≤500)"
        elif total_lines <= 1500:
            return "Good (501-1500)"
        elif total_lines <= 2000:
            return "Large (1501-2000)"
        else:
            return "Too Large (>2000)"

    def get_size_distribution(self) -> Tuple[int, int, int, int]:
        """Calculate PR size distribution counts.

        Returns:
            Tuple of (ideal_count, good_count, large_count, too_large_count)
        """
        if not self.is_loaded:
            return 0, 0, 0, 0

        ideal_count = len(self.df[self.df["total_lines"] <= 500])
        good_count = len(
            self.df[
                (self.df["total_lines"] > 500)
                & (self.df["total_lines"] <= 1500)
            ]
        )
        large_count = len(
            self.df[
                (self.df["total_lines"] > 1500)
                & (self.df["total_lines"] <= 2000)
            ]
        )
        too_large_count = len(self.df[self.df["total_lines"] > 2000])
        return ideal_count, good_count, large_count, too_large_count

    def get_averages(self) -> Tuple[float, float]:
        """Calculate average metrics.

        Returns:
            Tuple of (avg_lines, avg_files)
        """
        if not self.is_loaded:
            return 0.0, 0.0

        avg_lines = self.df["total_lines"].mean()
        avg_files = self.df["changed_files"].mean()
        return avg_lines, avg_files

    def get_recent_trends(self, count: int = 30) -> Tuple[float, float]:
        """Get recent trends from last N PRs.

        Args:
            count: Number of recent PRs to analyze (default: 30)

        Returns:
            Tuple of (recent_avg_lines, recent_avg_files)
        """
        if not self.is_loaded:
            return 0.0, 0.0

        recent_df = self.df.tail(count) if len(self.df) >= count else self.df
        recent_avg_lines = (
            recent_df["total_lines"].mean() if not recent_df.empty else 0
        )
        recent_avg_files = (
            recent_df["changed_files"].mean() if not recent_df.empty else 0
        )
        return recent_avg_lines, recent_avg_files

    def get_top_contributors(self, count: int = 10) -> pd.Series:
        """Get top contributors by PR count.

        Args:
            count: Number of top contributors to return

        Returns:
            pd.Series: Series with contributor names as index and PR counts as values
        """
        if not self.is_loaded:
            return pd.Series(dtype=object)

        return self.df["author"].value_counts().head(count)

    def get_large_prs(
        self, min_lines: int = 1500, count: int = 10
    ) -> pd.DataFrame:
        """Get recent large PRs above threshold.

        Args:
            min_lines: Minimum lines to be considered "large"
            count: Number of recent large PRs to return

        Returns:
            pd.DataFrame: DataFrame of large PRs
        """
        if not self.is_loaded:
            return pd.DataFrame()

        return self.df[self.df["total_lines"] > min_lines].tail(count)

    def get_daily_averages(self) -> pd.DataFrame:
        """Get daily average PR sizes.

        Returns:
            pd.DataFrame: DataFrame with date and average total_lines
        """
        if not self.is_loaded:
            return pd.DataFrame()

        # Group by date and calculate average size
        daily_avg = (
            self.df.groupby(self.df["date"].dt.date)["total_lines"]
            .mean()
            .reset_index()
        )
        daily_avg["date"] = pd.to_datetime(daily_avg["date"])
        return daily_avg

    def get_statistics_summary(self) -> Dict:
        """Get comprehensive statistics summary.

        Returns:
            Dict: Dictionary containing all key statistics
        """
        if not self.is_loaded:
            return {
                "total_prs": 0,
                "avg_lines": 0.0,
                "avg_files": 0.0,
                "ideal_count": 0,
                "good_count": 0,
                "large_count": 0,
                "too_large_count": 0,
                "recent_avg_lines": 0.0,
                "recent_avg_files": 0.0,
            }

        total_prs = len(self.df)
        avg_lines, avg_files = self.get_averages()
        ideal_count, good_count, large_count, too_large_count = (
            self.get_size_distribution()
        )
        recent_avg_lines, recent_avg_files = self.get_recent_trends()

        return {
            "total_prs": total_prs,
            "avg_lines": avg_lines,
            "avg_files": avg_files,
            "ideal_count": ideal_count,
            "good_count": good_count,
            "large_count": large_count,
            "too_large_count": too_large_count,
            "recent_avg_lines": recent_avg_lines,
            "recent_avg_files": recent_avg_files,
        }
