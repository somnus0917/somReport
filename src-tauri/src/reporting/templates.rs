pub const TEMPLATE_DEFAULT: &str = "default";
pub const TEMPLATE_DETAILED: &str = "detailed";
pub const TEMPLATE_EXECUTIVE: &str = "executive";

const DEFAULT_PROMPT: &str = "Generate a concise daily work report in Markdown. Group activities by category with time ranges. Include a brief Highlights section at the end with 2-3 key takeaways. Keep it professional and factual.";

const DETAILED_PROMPT: &str = "Generate a detailed daily work report in Markdown. For each activity include the time range, summary, detail (if available), and confidence score. Group by category. Include a Summary section with total time, work vs non-work breakdown, and category distribution. End with Highlights section.";

const EXECUTIVE_PROMPT: &str = "Generate a brief executive summary of the day's work in Markdown. Focus on high-level themes, major accomplishments, and time allocation across categories. No more than 10 lines. Use bullet points for clarity.";

pub fn get_template_prompt(template_id: &str) -> Option<&'static str> {
    match template_id {
        TEMPLATE_DEFAULT => Some(DEFAULT_PROMPT),
        TEMPLATE_DETAILED => Some(DETAILED_PROMPT),
        TEMPLATE_EXECUTIVE => Some(EXECUTIVE_PROMPT),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_templates_return_prompts() {
        assert!(get_template_prompt("default").is_some());
        assert!(get_template_prompt("detailed").is_some());
        assert!(get_template_prompt("executive").is_some());
    }

    #[test]
    fn test_unknown_template_returns_none() {
        assert!(get_template_prompt("nonexistent").is_none());
        assert!(get_template_prompt("").is_none());
    }
}
