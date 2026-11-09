//! # 文本处理工具
//!
//! 笔记摘要、字数统计与全局搜索片段共用同一套处理逻辑：
//! 1. 去掉常见 Markdown 标记（标题、列表、引用、代码符号等）；
//! 2. 折叠空白行，得到适合列表展示的纯文本；
//! 3. 按字符数截断，避免按字节截断导致中文乱码。

/// 统计正文字符数（忽略空白，与前端 `countCharacters` 保持一致）
pub fn count_characters(text: &str) -> i64 {
    text.chars().filter(|ch| !ch.is_whitespace()).count() as i64
}

/// 按字符截断，超出时追加省略号
pub fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }

    let mut truncated: String = text.chars().take(limit).collect();
    truncated.push('…');
    truncated
}

/// 把 Markdown 正文转换成用于摘要 / 搜索的一行纯文本
///
/// `limit` 是收集阶段的上限；达到上限后停止继续拼接，避免搜索大笔记时
/// 把整篇正文都清洗一遍。
fn plain_text(content: &str, limit: usize) -> String {
    let mut buffer = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // 纯分隔线（--- / *** / ___）没有摘要价值
        if trimmed.chars().count() >= 3
            && trimmed
                .chars()
                .all(|ch| matches!(ch, '-' | '*' | '_' | '='))
        {
            continue;
        }

        let stripped = trimmed.trim_start_matches(|ch: char| {
            matches!(ch, '#' | '>' | '-' | '*' | '+' | '|' | '`' | ' ')
        });
        let cleaned: String = stripped
            .chars()
            .filter(|ch| !matches!(ch, '`' | '*'))
            .collect();
        let cleaned = cleaned.trim();
        if cleaned.is_empty() {
            continue;
        }

        if !buffer.is_empty() {
            buffer.push(' ');
        }
        buffer.push_str(cleaned);

        if buffer.chars().count() >= limit {
            break;
        }
    }

    buffer
}

/// 生成列表摘要：去掉常见 Markdown 标记、折叠空白并截断
pub fn excerpt(content: &str, limit: usize) -> String {
    let buffer = plain_text(content, limit.max(1) * 2);
    truncate_chars(&buffer, limit)
}

/// 在字符序列中查找子串，返回起始下标（用于保持中文下标安全）
fn find_chars(haystack: &[char], needle: &[char]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// 生成包含关键字的搜索片段
///
/// 关键字命中正文时截取命中位置附近的文本，并在两端补省略号；
/// 未命中（例如只命中标题）时退化为从正文开头生成的普通摘要。
pub fn match_excerpt(content: &str, keyword: &str, limit: usize) -> String {
    let limit = limit.max(8);
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return excerpt(content, limit);
    }

    let plain = plain_text(content, 20_000);
    let chars: Vec<char> = plain.chars().collect();
    let needle: Vec<char> = keyword.chars().collect();

    let Some(position) = find_chars(&chars, &needle) else {
        return truncate_chars(plain.trim(), limit);
    };

    // 让命中位置大致落在片段前三分之一处，保留一点上下文
    let context = limit / 3;
    let mut start = position.saturating_sub(context);
    let mut end = (start + limit).min(chars.len());
    if end - start < limit {
        start = end.saturating_sub(limit);
    }
    if end < chars.len() {
        // 尽量在遇到空格时结束片段，避免把英文单词截断
        if let Some(offset) = chars[start..end].iter().rposition(|ch| ch.is_whitespace()) {
            if offset >= limit / 2 {
                end = start + offset;
            }
        }
    }

    let mut snippet: String = chars[start..end].iter().collect();
    snippet = snippet.trim().to_string();

    if start > 0 {
        snippet.insert(0, '…');
    }
    if end < chars.len() {
        snippet.push('…');
    }

    snippet
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excerpt_strips_markdown_and_truncates() {
        let text = excerpt("# 标题\n\n**重点** 内容 `code`\n\n---\n第二段", 80);
        assert!(!text.contains('#'));
        assert!(!text.contains('*'));
        assert!(!text.contains('`'));
        assert!(text.contains("重点"));
        assert!(text.contains("第二段"));

        let long = excerpt(&"字".repeat(300), 20);
        assert_eq!(long.chars().count(), 21);
        assert!(long.ends_with('…'));

        assert_eq!(excerpt("   \n\n", 20), "");
    }

    #[test]
    fn character_count_ignores_whitespace() {
        assert_eq!(count_characters(" 一 二\n三\t"), 3);
        assert_eq!(count_characters(""), 0);
    }

    #[test]
    fn match_excerpt_keeps_keyword_and_adds_ellipsis() {
        let content = format!("{} 关键字在这里 {}", "前缀".repeat(60), "后缀".repeat(60));
        let snippet = match_excerpt(&content, "关键字", 40);

        assert!(snippet.contains("关键字"));
        assert!(snippet.starts_with('…'));
        assert!(snippet.ends_with('…'));
        assert!(snippet.chars().count() <= 42);
    }

    #[test]
    fn match_excerpt_falls_back_to_beginning() {
        let snippet = match_excerpt("**开头** 的内容", "不存在", 20);
        assert_eq!(snippet, "开头 的内容");

        let long = match_excerpt(&"字".repeat(100), "不存在", 20);
        assert_eq!(long.chars().count(), 21);
        assert!(long.ends_with('…'));
    }

    #[test]
    fn match_excerpt_handles_empty_content_and_keyword() {
        assert_eq!(match_excerpt("", "关键字", 20), "");
        assert_eq!(match_excerpt("正文内容", "   ", 20), "正文内容");
    }
}
