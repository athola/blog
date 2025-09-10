# Scripts

This directory contains utility scripts for the project.

## PR Size Trend Visualization

The `visualize_pr_trends.py` script generates visualizations from PR size metrics data.

### Dependencies

- Python 3.7+
- pandas
- matplotlib
- seaborn
- numpy

Install dependencies with:
```bash
pip install -r requirements.txt
```

Or use `uv` (recommended):
```bash
uv pip install -r requirements.txt
```

### Usage

```bash
python3 visualize_pr_trends.py ../.github/metrics/pr-size-history.csv
```

Or with `uv` (recommended):
```bash
uv run visualize_pr_trends.py ../.github/metrics/pr-size-history.csv
```

### Output

The script generates:
1. Size distribution pie chart
2. PR size trend line chart
3. Top contributors bar chart
4. Markdown report with statistics

All outputs are saved to the `.github/reports/` directory:
- Visualizations are treated as artifacts and not committed to the repository
- The markdown report is committed for documentation purposes