//! 数值工具
//!
//! 纯函数，无外部依赖、无日志 —— 由调用方决定告警与降级策略。

/// 余弦相似度
///
/// 维度不一致时返回 `None`：不同维度（乃至不同模型）的向量空间不可比，
/// 禁止用 `zip` 静默截断到较短一方产生假分数。调用方应跳过该向量并告警。
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a <= f32::EPSILON || norm_b <= f32::EPSILON {
        Some(0.0)
    } else {
        Some(dot / (norm_a * norm_b))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_vectors_similarity_is_one() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v).unwrap();
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn orthogonal_vectors_similarity_is_zero() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        assert_eq!(cosine_similarity(&a, &b), Some(0.0));
    }

    #[test]
    fn dimension_mismatch_returns_none() {
        // 384 vs 512 —— 必须拒绝，而非截断到 384 维算出一个假分数
        assert_eq!(cosine_similarity(&[0.1; 384], &[0.1; 512]), None);
    }

    #[test]
    fn empty_vector_returns_none() {
        assert_eq!(cosine_similarity(&[], &[]), None);
    }

    #[test]
    fn zero_vector_similarity_is_zero() {
        assert_eq!(cosine_similarity(&[0.0; 4], &[1.0; 4]), Some(0.0));
    }
}

// ============================================================
// 属性测试（第一层：物理不变量）
// ============================================================

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    fn vec_of_len(len: usize) -> impl Strategy<Value = Vec<f32>> {
        prop::collection::vec(-10.0f32..10.0f32, len)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// 不变量：任意输入下余弦 ∈ [-1, 1]，永不出界
        #[test]
        fn cosine_always_within_unit_interval(a in vec_of_len(8), b in vec_of_len(8)) {
            if let Some(c) = cosine_similarity(&a, &b) {
                prop_assert!(
                    (-1.0 - 1e-6..=1.0 + 1e-6).contains(&c),
                    "cosine out of range: {}", c
                );
            }
        }

        /// 不变量：cos(a,b) == cos(b,a)
        #[test]
        fn cosine_is_symmetric(a in vec_of_len(8), b in vec_of_len(8)) {
            match (cosine_similarity(&a, &b), cosine_similarity(&b, &a)) {
                (Some(x), Some(y)) => prop_assert!((x - y).abs() < 1e-5, "{} vs {}", x, y),
                (None, None) => {}
                _ => prop_assert!(false, "对称性被破坏：一侧 None 一侧 Some"),
            }
        }

        /// 不变量：维度不一致永远是 None，绝不静默截断出假分数
        #[test]
        fn dimension_mismatch_never_returns_score(a in vec_of_len(8), b in vec_of_len(9)) {
            prop_assert_eq!(cosine_similarity(&a, &b), None);
        }

        /// 不变量：非零向量自相似度恒为 1
        #[test]
        fn self_similarity_is_one(
            v in vec_of_len(8).prop_filter("非零向量", |v| v.iter().any(|x| x.abs() > 1e-6))
        ) {
            let c = cosine_similarity(&v, &v).unwrap();
            prop_assert!((c - 1.0).abs() < 1e-4, "cos(v,v) = {}", c);
        }
    }
}
