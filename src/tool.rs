use std::iter;

use nom::InputIter;
use rand::{Rng, distributions::Alphanumeric, thread_rng};

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

    if exp.len() <= 0 { return value.len() <= 0; }

    let mut exp_chars = exp.chars();

    let mut match_curr_char = exp_chars.next();
    let mut wildcard_state = 0;

    for letter in value.iter_elements() {

        if match_curr_char.is_some() && match_curr_char.unwrap() == '*' {
            wildcard_state = u16::MAX;
            match_curr_char = exp_chars.next();
        } else if match_curr_char.is_some() && match_curr_char.unwrap() == '?' {
            wildcard_state = 1;
            match_curr_char = exp_chars.next();
        }

        if match_curr_char.is_some() && match_curr_char.unwrap() == letter {
            // 先把通配符的状态恢复到 0
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

    if match_curr_char.is_some() && match_curr_char.unwrap() !=  '*' {
        return false;
    }

    exp_chars.next().is_none()
}

#[test]
fn test_fuzzy_search() {
    
    // 正常的匹配测试
    assert!(fuzzy_search("*.info", "mrxzx.info"));
    assert!(fuzzy_search("L?uYuK?n", "LiuYuKun"));

    // 错误的匹配测试
    assert!(!fuzzy_search("*.com", "mrxzx.info"));
    assert!(!fuzzy_search("?.com", "baidu.com"));
    assert!(!fuzzy_search("dorea-server", "dorea-ser"));
    assert!(!fuzzy_search("dorea-ser", "dorea-server"));
    assert!(!fuzzy_search("dorea?", "dorea"));
    assert!(fuzzy_search("dorea*", "dorea"));

}