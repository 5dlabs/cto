//! Digest content generator.
//!
//! Builds email content from research entries with AI analysis.

use chrono::{DateTime, Utc};
use std::fmt::Write;

use super::analyzer::{ActionItem, DigestAnalysis};
use crate::storage::IndexEntry;

/// Generates digest email content from research entries.
pub struct DigestGenerator;

impl DigestGenerator {
    /// Generate HTML email content with AI analysis.
    #[must_use]
    pub fn generate_html(
        entries: &[&IndexEntry],
        analysis: Option<&DigestAnalysis>,
        generated_at: DateTime<Utc>,
    ) -> String {
        let entry_count = entries.len();
        let date_str = generated_at.format("%B %d, %Y").to_string();

        // Build AI analysis section
        let analysis_html = if let Some(a) = analysis {
            Self::build_analysis_html(a)
        } else {
            String::from(r#"<div class="section"><p class="muted">AI analysis not available</p></div>"#)
        };

        // Build entries section
        let mut entries_html = String::new();
        for entry in entries {
            let categories: Vec<_> = entry.categories.iter().map(ToString::to_string).collect();
            let category_tags = categories
                .iter()
                .map(|c| format!(r#"<span class="tag">{c}</span>"#))
                .collect::<Vec<_>>()
                .join(" ");

            let score_color = if entry.score >= 0.8 {
                "#22c55e" // green
            } else if entry.score >= 0.6 {
                "#eab308" // yellow
            } else {
                "#6b7280" // gray
            };

            let _ = write!(
                entries_html,
                r#"
        <div class="entry">
            <div class="entry-header">
                <span class="author">@{author}</span>
                <span class="score" style="color: {score_color};">
                    {score:.2}
                </span>
            </div>
            <p class="preview">{preview}</p>
            <div class="tags">{category_tags}</div>
        </div>
"#,
                author = entry.author,
                score = entry.score,
                score_color = score_color,
                preview = html_escape(&entry.preview),
                category_tags = category_tags,
            );
        }

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', sans-serif;
            line-height: 1.6;
            color: #e5e7eb;
            background-color: #0f172a;
            margin: 0;
            padding: 20px;
        }}
        .container {{
            max-width: 700px;
            margin: 0 auto;
            background: #1e293b;
            border-radius: 12px;
            overflow: hidden;
            border: 1px solid #334155;
        }}
        .header {{
            background: linear-gradient(135deg, #0ea5e9 0%, #06b6d4 100%);
            color: white;
            padding: 28px;
        }}
        .header h1 {{
            margin: 0 0 8px 0;
            font-size: 26px;
            font-weight: 700;
        }}
        .header .subtitle {{
            opacity: 0.9;
            font-size: 14px;
        }}
        .content {{
            padding: 24px;
        }}
        .summary-box {{
            background: #0f172a;
            border: 1px solid #334155;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 24px;
        }}
        .summary-stat {{
            font-size: 36px;
            font-weight: 700;
            color: #0ea5e9;
        }}
        .summary-label {{
            color: #94a3b8;
            font-size: 14px;
        }}
        .section {{
            margin-bottom: 28px;
        }}
        .section-title {{
            font-size: 18px;
            font-weight: 600;
            color: #f1f5f9;
            margin: 0 0 16px 0;
            padding-bottom: 8px;
            border-bottom: 1px solid #334155;
        }}
        .section-title .icon {{
            margin-right: 8px;
        }}
        .action-item {{
            background: #0f172a;
            border: 1px solid #334155;
            border-radius: 8px;
            padding: 16px;
            margin-bottom: 12px;
        }}
        .action-item:last-child {{
            margin-bottom: 0;
        }}
        .action-title {{
            font-weight: 600;
            color: #f1f5f9;
            margin-bottom: 8px;
        }}
        .action-desc {{
            color: #94a3b8;
            font-size: 14px;
            margin-bottom: 10px;
        }}
        .action-meta {{
            display: flex;
            gap: 12px;
            font-size: 12px;
        }}
        .badge {{
            display: inline-block;
            padding: 2px 8px;
            border-radius: 4px;
            font-weight: 500;
        }}
        .badge-effort {{
            background: #1e3a5f;
            color: #38bdf8;
        }}
        .badge-impact {{
            background: #14532d;
            color: #4ade80;
        }}
        .badge-high {{
            background: #7f1d1d;
            color: #fca5a5;
        }}
        .tip {{
            background: #0f172a;
            border-left: 3px solid #0ea5e9;
            padding: 12px 16px;
            margin-bottom: 8px;
            font-size: 14px;
            color: #cbd5e1;
        }}
        .recommendation {{
            background: linear-gradient(135deg, #0c4a6e 0%, #164e63 100%);
            border-radius: 8px;
            padding: 16px;
            color: #e0f2fe;
            font-size: 15px;
        }}
        .divider {{
            height: 1px;
            background: #334155;
            margin: 28px 0;
        }}
        .entries-title {{
            font-size: 16px;
            font-weight: 600;
            color: #94a3b8;
            margin: 0 0 16px 0;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}
        .entry {{
            background: #0f172a;
            border: 1px solid #334155;
            border-radius: 8px;
            padding: 14px;
            margin-bottom: 10px;
        }}
        .entry:last-child {{
            margin-bottom: 0;
        }}
        .entry-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;
        }}
        .author {{
            font-weight: 600;
            color: #0ea5e9;
        }}
        .score {{
            font-size: 13px;
            font-weight: 600;
        }}
        .preview {{
            margin: 8px 0;
            color: #cbd5e1;
            font-size: 14px;
        }}
        .tags {{
            margin-top: 8px;
        }}
        .tag {{
            display: inline-block;
            background: #1e3a5f;
            color: #7dd3fc;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 11px;
            margin-right: 4px;
        }}
        .footer {{
            background: #0f172a;
            padding: 16px 24px;
            text-align: center;
            font-size: 12px;
            color: #64748b;
            border-top: 1px solid #334155;
        }}
        .footer a {{
            color: #0ea5e9;
            text-decoration: none;
        }}
        .muted {{
            color: #64748b;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔬 CTO Research Digest</h1>
            <div class="subtitle">{date_str}</div>
        </div>
        <div class="content">
            <div class="summary-box">
                <div class="summary-stat">{entry_count}</div>
                <div class="summary-label">new research entries analyzed</div>
            </div>

            {analysis_html}

            <div class="divider"></div>

            <h3 class="entries-title">📚 Source Entries</h3>
            {entries_html}
        </div>
        <div class="footer">
            CTO Research Pipeline &bull;
            <a href="https://github.com/5dlabs/cto">github.com/5dlabs/cto</a>
        </div>
    </div>
</body>
</html>"#,
            date_str = date_str,
            entry_count = entry_count,
            analysis_html = analysis_html,
            entries_html = entries_html,
        )
    }

    /// Build HTML for the AI analysis section.
    fn build_analysis_html(analysis: &DigestAnalysis) -> String {
        let mut html = String::new();

        // Summary
        let _ = write!(
            html,
            r#"
            <div class="section">
                <h2 class="section-title"><span class="icon">📊</span>Summary</h2>
                <p style="color: #cbd5e1; margin: 0;">{summary}</p>
            </div>
"#,
            summary = html_escape(&analysis.summary),
        );

        // High Priority Actions
        if !analysis.high_priority.is_empty() {
            let _ = write!(
                html,
                r#"
            <div class="section">
                <h2 class="section-title"><span class="icon">🚀</span>High Priority - Build This</h2>
                {items}
            </div>
"#,
                items = Self::build_action_items_html(&analysis.high_priority),
            );
        }

        // Worth Investigating
        if !analysis.worth_investigating.is_empty() {
            let _ = write!(
                html,
                r#"
            <div class="section">
                <h2 class="section-title"><span class="icon">🔍</span>Worth Investigating</h2>
                {items}
            </div>
"#,
                items = Self::build_action_items_html(&analysis.worth_investigating),
            );
        }

        // Tips & Tricks
        if !analysis.tips_and_tricks.is_empty() {
            let tips_html: String = analysis
                .tips_and_tricks
                .iter()
                .map(|tip| format!(r#"<div class="tip">💡 {}</div>"#, html_escape(tip)))
                .collect::<Vec<_>>()
                .join("\n");

            let _ = write!(
                html,
                r#"
            <div class="section">
                <h2 class="section-title"><span class="icon">⚡</span>Tips & Tricks</h2>
                {tips_html}
            </div>
"#,
                tips_html = tips_html,
            );
        }

        // Overall Recommendation
        let _ = write!(
            html,
            r#"
            <div class="section">
                <div class="recommendation">
                    <strong>📌 Bottom Line:</strong> {recommendation}
                </div>
            </div>
"#,
            recommendation = html_escape(&analysis.overall_recommendation),
        );

        html
    }

    /// Build HTML for action items list.
    fn build_action_items_html(items: &[ActionItem]) -> String {
        items
            .iter()
            .map(|item| {
                let effort_class = if item.effort == "high" {
                    "badge-high"
                } else {
                    "badge-effort"
                };
                let impact_class = if item.impact == "high" {
                    "badge-impact"
                } else {
                    "badge-effort"
                };

                format!(
                    r#"<div class="action-item">
                    <div class="action-title">{title}</div>
                    <div class="action-desc">{description}</div>
                    <div class="action-meta">
                        <span class="badge {effort_class}">Effort: {effort}</span>
                        <span class="badge {impact_class}">Impact: {impact}</span>
                    </div>
                </div>"#,
                    title = html_escape(&item.title),
                    description = html_escape(&item.description),
                    effort = item.effort,
                    impact = item.impact,
                    effort_class = effort_class,
                    impact_class = impact_class,
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate plain-text email content with AI analysis.
    #[must_use]
    pub fn generate_text(
        entries: &[&IndexEntry],
        analysis: Option<&DigestAnalysis>,
        generated_at: DateTime<Utc>,
    ) -> String {
        let date_str = generated_at.format("%B %d, %Y").to_string();

        let mut text = format!(
            "CTO Research Digest
{date_str}

{count} new research entries analyzed
================================================================================

",
            date_str = date_str,
            count = entries.len(),
        );

        // Add AI analysis
        if let Some(a) = analysis {
            text.push_str("📊 SUMMARY\n");
            text.push_str(&a.summary);
            text.push_str("\n\n");

            if !a.high_priority.is_empty() {
                text.push_str("🚀 HIGH PRIORITY - BUILD THIS\n");
                text.push_str(&"-".repeat(40));
                text.push('\n');
                for item in &a.high_priority {
                    let _ = write!(
                        text,
                        "• {title}\n  {desc}\n  Effort: {effort} | Impact: {impact}\n\n",
                        title = item.title,
                        desc = item.description,
                        effort = item.effort,
                        impact = item.impact,
                    );
                }
            }

            if !a.worth_investigating.is_empty() {
                text.push_str("🔍 WORTH INVESTIGATING\n");
                text.push_str(&"-".repeat(40));
                text.push('\n');
                for item in &a.worth_investigating {
                    let _ = write!(
                        text,
                        "• {title}\n  {desc}\n  Effort: {effort} | Impact: {impact}\n\n",
                        title = item.title,
                        desc = item.description,
                        effort = item.effort,
                        impact = item.impact,
                    );
                }
            }

            if !a.tips_and_tricks.is_empty() {
                text.push_str("⚡ TIPS & TRICKS\n");
                text.push_str(&"-".repeat(40));
                text.push('\n');
                for tip in &a.tips_and_tricks {
                    let _ = writeln!(text, "💡 {tip}");
                }
                text.push('\n');
            }

            text.push_str("📌 BOTTOM LINE\n");
            text.push_str(&a.overall_recommendation);
            text.push_str("\n\n");
        }

        text.push_str(&"=".repeat(80));
        text.push_str("\n📚 SOURCE ENTRIES\n");
        text.push_str(&"=".repeat(80));
        text.push_str("\n\n");

        for entry in entries {
            let categories: Vec<_> = entry.categories.iter().map(ToString::to_string).collect();

            let _ = write!(
                text,
                "@{author} (Score: {score:.2})
{preview}
Categories: {categories}

---

",
                author = entry.author,
                score = entry.score,
                preview = entry.preview,
                categories = categories.join(", "),
            );
        }

        text.push_str(
            "
---
CTO Research Pipeline
https://github.com/5dlabs/cto
",
        );

        text
    }
}

/// Simple HTML escaping for user content.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }
}
