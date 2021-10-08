use std::iter;

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