use std::iter;

use nom::InputIter;
use rand::{distributions::Alphanumeric, thread_rng, Rng};

// 生成随机字符串
pub fn rand_str() -> String {
    let mut rng = thread_rng();

    let chars: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(15)
        .collect();

    chars
}

// 模糊匹配单条匹配函数
pub fn fuzzy_search(exp: &str, value: &str) -> bool {
    if exp.is_empty() {
        return true;
    }

    // 处理 ^ 和 $ 锚点
    let has_prefix = exp.starts_with('^');
    let has_suffix = exp.ends_with('$');
    let inner = match (has_prefix, has_suffix) {
        (true, true) => &exp[1..exp.len() - 1],
        (true, false) => &exp[1..],
        (false, true) => &exp[..exp.len() - 1],
        (false, false) => exp,
    };

    // 无通配符 → 简单字符串匹配
    if !inner.contains('*') && !inner.contains('?') {
        if inner.is_empty() {
            return true;
        }
        return match (has_prefix, has_suffix) {
            (true, true) => value == inner,
            (true, false) => value.starts_with(inner),
            (false, true) => value.ends_with(inner),
            (false, false) => value.contains(inner),
        };
    }

    // 通配符匹配（原有逻辑）
    let mut exp_chars = inner.chars();
    let mut match_curr_char = exp_chars.next();
    let mut wildcard_state: u16 = 0;

    for letter in value.iter_elements() {
        if match_curr_char.is_some() && match_curr_char.unwrap() == '*' {
            wildcard_state = u16::MAX;
            match_curr_char = exp_chars.next();
        } else if match_curr_char.is_some() && match_curr_char.unwrap() == '?' {
            wildcard_state = 1;
            match_curr_char = exp_chars.next();
        }

        if match_curr_char.is_some() && match_curr_char.unwrap() == letter {
            wildcard_state = 0;
            match_curr_char = exp_chars.next();
        } else if wildcard_state > 0 {
            wildcard_state -= 1;
        } else {
            return false;
        }
    }

    // * 可以不被匹配，但 ? 不行！
    if wildcard_state == 1 {
        return false;
    }

    if match_curr_char.is_some() && match_curr_char.unwrap() != '*' {
        return false;
    }

    exp_chars.next().is_none()
}

#[test]
fn test_fuzzy_search() {
    // 旧的通配符匹配
    assert!(fuzzy_search("*.info", "mrxzx.info"));
    assert!(fuzzy_search("L?uYuK?n", "LiuYuKun"));
    assert!(!fuzzy_search("*.com", "mrxzx.info"));
    assert!(!fuzzy_search("?.com", "baidu.com"));
    assert!(!fuzzy_search("dorea-server", "dorea-ser"));
    assert!(!fuzzy_search("dorea-ser", "dorea-server"));
    assert!(!fuzzy_search("dorea?", "dorea"));
    assert!(fuzzy_search("dorea*", "dorea"));

    // 新：默认子串匹配
    assert!(fuzzy_search("admin", "user:admin"));
    assert!(fuzzy_search("user", "user:admin"));
    assert!(!fuzzy_search("admin", "user:root"));

    // 新：^ 前缀匹配
    assert!(fuzzy_search("^user", "user:admin"));
    assert!(!fuzzy_search("^admin", "user:admin"));

    // 新：$ 后缀匹配
    assert!(fuzzy_search(".log$", "error.log"));
    assert!(!fuzzy_search(".log$", "log.txt"));

    // 新：^...$ 精确匹配
    assert!(fuzzy_search("^hello$", "hello"));
    assert!(!fuzzy_search("^hello$", "hello!"));
}
