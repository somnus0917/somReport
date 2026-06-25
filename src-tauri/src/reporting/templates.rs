pub const TEMPLATE_DEFAULT: &str = "default";
pub const TEMPLATE_DETAILED: &str = "detailed";
pub const TEMPLATE_EXECUTIVE: &str = "executive";

const DEFAULT_PROMPT: &str = "生成一份简洁的每日工作报告，使用 Markdown 格式。按类别分组活动，附带时间范围。末尾包含一个简短的「亮点」部分，列出 2-3 个关键要点。保持专业和事实性。全部使用中文。";

const DETAILED_PROMPT: &str = "生成一份详细的每日工作报告，使用 Markdown 格式。每个活动包含时间范围、摘要、详情（如有）和置信度分数。按类别分组。包含「摘要」部分，列出总时间、工作与非工作时间分布、类别分布。末尾是「亮点」部分。全部使用中文。";

const EXECUTIVE_PROMPT: &str = "生成一份简短的每日工作高管摘要，使用 Markdown 格式。聚焦于高层次主题、主要成就和各类别的时间分配。不超过 10 行。使用要点符号提高清晰度。全部使用中文。";

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
