# PR Size Metrics

This directory contains historical data and reports for PR size metrics.

## Structure

- `pr-size-history.csv` - Historical data of all merged PRs with their size metrics
- `pr_size_trend.md` - Generated trend report with statistics and insights

## Collected Metrics

For each merged PR, we collect:
- Date merged
- PR number and title
- Author
- Total lines changed (additions + deletions)
- Additions
- Deletions
- Files changed
- Size category (ideal, good, large, too-large)
- PR URL

## Workflows

1. **PR Size Metrics Collection** (`pr-size-metrics.yml`)
   - Runs when PRs are merged to master
   - Collects size metrics and appends to the history CSV

2. **PR Size Trend Report** (`pr-size-trend-report.yml`)
   - Runs daily to generate trend analysis
   - Creates a summary report with:
     - Size distribution statistics
     - Averages over time
     - Recent trends
     - Top contributors
     - Large PRs requiring attention

## Size Categories

- ✅ **Ideal**: ≤ 500 lines changed
- 🟡 **Good**: 501-1500 lines changed
- ⚠️ **Large**: 1501-2000 lines changed
- ❌ **Too Large**: > 2000 lines changed