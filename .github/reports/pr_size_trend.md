# PR Size Trend Report

Generated on: 2025-09-10 00:58:27 UTC

## Summary Statistics

- **Total PRs analyzed**: 3
- **Average lines changed per PR**: 1166.7
- **Average files changed per PR**: 21.7

## PR Size Distribution

| Size Category | Count | Percentage |
|---------------|-------|------------|
| ✅ Ideal (≤500 lines) | 1 | 33.3% |
| 🟡 Good (501-1500 lines) | 1 | 33.3% |
| ⚠️ Large (1501-2000 lines) | 0 | 0.0% |
| ❌ Too Large (>2000 lines) | 1 | 33.3% |

## Recent Trends (Last 30 PRs)

- **Recent average lines changed per PR**: 1166.7
- **Recent average files changed per PR**: 21.7


## Top Contributors (By PR Count)

| Contributor | PR Count | Avg Size |
|-------------|----------|----------|
| bob | 2 | 1525.0 |
| alice | 1 | 450.0 |

## Recent Large PRs (>1500 lines)

| PR | Title | Author | Lines | Date |
|----|-------|--------|-------|------|
| [#106](https://github.com/example/repo/pull/106) | Add comprehensive testing | bob | 2200 | 2024-01-20 |

## Key Insights

⚠️ **PR sizing needs improvement**: Less than 40% of PRs are in the ideal size range.

🚨 **Too many large PRs**: 33.3% of PRs exceed the 2000-line limit. Consider breaking these down.

➡️ **Consistent PR sizing**: Recent average (1167 lines) is similar to overall average (1167 lines).


## Recommendations

1. **Focus on smaller PRs**: Aim to break down features into smaller, more focused changes.
2. **Use feature flags**: Consider using feature flags to ship incomplete features safely.
3. **Preparatory PRs**: Create separate PRs for infrastructure changes before feature implementation.

1. **Review large PR process**: Establish a review process for PRs over 1500 lines.
2. **Breaking down strategies**: Document strategies for splitting large changes.
3. **Reviewer assignment**: Assign multiple reviewers for large PRs to ensure thorough review.

1. **Recent trend concern**: Recent PRs are averaging over 1000 lines - consider smaller iterations.
2. **Sprint planning**: Break down larger features at the planning stage.

### General Best Practices

- **Single responsibility**: Each PR should address one specific concern
- **Incremental changes**: Build features incrementally with each PR adding value
- **Review guidelines**: Establish clear guidelines for PR review based on size
- **Automation**: Use automated checks to enforce size limits where appropriate

