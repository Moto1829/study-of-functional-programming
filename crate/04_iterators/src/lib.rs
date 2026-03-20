//! # 第4章: イテレータと遅延評価
//!
//! Rust の `Iterator` トレイトを中心に、関数型スタイルのデータ変換を学ぶ。
//!
//! - 基本アダプタ (`map`, `filter`, `fold`, `flat_map`, `zip`, `chain`)
//! - 無限イテレータと `take`
//! - データ変換パイプライン
//! - カスタムイテレータの実装
//! - `scan`, `take_while`, `skip_while`

// ============================================================================
// 1. map / filter / fold の基本使用例
// ============================================================================

/// 与えられたスライスの各要素を2乗した `Vec` を返す。
///
/// # Examples
///
/// ```
/// use iterators::square_all;
/// assert_eq!(square_all(&[1, 2, 3, 4]), vec![1, 4, 9, 16]);
/// ```
pub fn square_all(numbers: &[i32]) -> Vec<i32> {
    numbers.iter().map(|&x| x * x).collect()
}

/// 与えられたスライスから偶数のみを抽出した `Vec` を返す。
///
/// # Examples
///
/// ```
/// use iterators::filter_evens;
/// assert_eq!(filter_evens(&[1, 2, 3, 4, 5, 6]), vec![2, 4, 6]);
/// ```
pub fn filter_evens(numbers: &[i32]) -> Vec<i32> {
    numbers.iter().copied().filter(|x| x % 2 == 0).collect()
}

/// `fold` を使ってスライスの合計を計算する。
///
/// # Examples
///
/// ```
/// use iterators::sum_with_fold;
/// assert_eq!(sum_with_fold(&[1, 2, 3, 4, 5]), 15);
/// assert_eq!(sum_with_fold(&[]), 0);
/// ```
pub fn sum_with_fold(numbers: &[i32]) -> i32 {
    numbers.iter().fold(0, |acc, &x| acc + x)
}

/// `fold` を使って文字列スライスを1つに連結する。
///
/// # Examples
///
/// ```
/// use iterators::join_with_fold;
/// assert_eq!(join_with_fold(&["Hello", ", ", "world", "!"]), "Hello, world!");
/// ```
pub fn join_with_fold(words: &[&str]) -> String {
    words.iter().fold(String::new(), |mut acc, &w| {
        acc.push_str(w);
        acc
    })
}

// ============================================================================
// 2. flat_map の使用例
// ============================================================================

/// 複数の文章を単語単位に分割してフラットに並べた `Vec` を返す。
///
/// `flat_map` を使うことで、各文章 → 単語列 への変換とフラット化を1ステップで行う。
///
/// # Examples
///
/// ```
/// use iterators::words_from_sentences;
/// let sentences = vec!["hello world", "foo bar"];
/// assert_eq!(words_from_sentences(&sentences), vec!["hello", "world", "foo", "bar"]);
/// ```
pub fn words_from_sentences(sentences: &[&str]) -> Vec<String> {
    sentences
        .iter()
        .flat_map(|s| s.split_whitespace())
        .map(|w| w.to_string())
        .collect()
}

/// 入れ子になった `Vec<Vec<T>>` を1次元の `Vec<T>` にフラット化する。
///
/// # Examples
///
/// ```
/// use iterators::flatten_nested;
/// let nested = vec![vec![1, 2, 3], vec![4, 5], vec![6]];
/// assert_eq!(flatten_nested(nested), vec![1, 2, 3, 4, 5, 6]);
/// ```
pub fn flatten_nested(nested: Vec<Vec<i32>>) -> Vec<i32> {
    nested.into_iter().flat_map(|v| v.into_iter()).collect()
}

// ============================================================================
// 3. zip / chain の使用例
// ============================================================================

/// 名前リストとスコアリストを `zip` してペアの `Vec` を返す。
///
/// 短い方のリストで打ち切られる。
///
/// # Examples
///
/// ```
/// use iterators::zip_names_scores;
/// let result = zip_names_scores(&["Alice", "Bob"], &[95, 87, 72]);
/// assert_eq!(result, vec![("Alice".to_string(), 95), ("Bob".to_string(), 87)]);
/// ```
pub fn zip_names_scores(names: &[&str], scores: &[i32]) -> Vec<(String, i32)> {
    names
        .iter()
        .zip(scores.iter())
        .map(|(&name, &score)| (name.to_string(), score))
        .collect()
}

/// 2つのスライスを `chain` で連結した `Vec` を返す。
///
/// # Examples
///
/// ```
/// use iterators::chain_slices;
/// assert_eq!(chain_slices(&[1, 2, 3], &[4, 5, 6]), vec![1, 2, 3, 4, 5, 6]);
/// ```
pub fn chain_slices(a: &[i32], b: &[i32]) -> Vec<i32> {
    a.iter().chain(b.iter()).copied().collect()
}

// ============================================================================
// 4. 無限イテレータと take（フィボナッチ数列）
// ============================================================================

/// 無限のフィボナッチ数列を生成するイテレータを返す。
///
/// `std::iter::from_fn` と状態変数を組み合わせ、
/// `take` や `take_while` で必要な分だけ消費する。
///
/// # Examples
///
/// ```
/// use iterators::fibonacci;
/// let first_eight: Vec<u64> = fibonacci().take(8).collect();
/// assert_eq!(first_eight, vec![0, 1, 1, 2, 3, 5, 8, 13]);
/// ```
pub fn fibonacci() -> impl Iterator<Item = u64> {
    let mut state = (0u64, 1u64);
    std::iter::from_fn(move || {
        let next = state.0;
        state = (state.1, state.0 + state.1);
        Some(next)
    })
}

/// 100 未満のフィボナッチ数をすべて返す。
///
/// # Examples
///
/// ```
/// use iterators::fibonacci_under_100;
/// let result = fibonacci_under_100();
/// assert_eq!(result, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);
/// ```
pub fn fibonacci_under_100() -> Vec<u64> {
    fibonacci().take_while(|&x| x < 100).collect()
}

/// `iter::repeat` と `cycle` の使用例。
///
/// - `repeat(x)`: 同じ値を無限に繰り返す
/// - `cycle()`: スライスを無限に繰り返す
///
/// # Examples
///
/// ```
/// use iterators::demonstrate_infinite_iterators;
/// let (repeated, cycled) = demonstrate_infinite_iterators();
/// assert_eq!(repeated, vec![42, 42, 42]);
/// assert_eq!(cycled, vec![1, 2, 3, 1, 2, 3, 1]);
/// ```
pub fn demonstrate_infinite_iterators() -> (Vec<i32>, Vec<i32>) {
    let repeated: Vec<i32> = std::iter::repeat(42).take(3).collect();
    let cycled: Vec<i32> = vec![1, 2, 3].into_iter().cycle().take(7).collect();
    (repeated, cycled)
}

// ============================================================================
// 5. イテレータチェーンによるデータ変換パイプライン
// ============================================================================

/// ログ行のリストを処理するパイプライン。
///
/// 処理ステップ:
/// 1. 空行を除外
/// 2. `#` で始まるコメント行を除外
/// 3. 前後の空白を削除
/// 4. 大文字に変換
///
/// # Examples
///
/// ```
/// use iterators::process_log_lines;
/// let log = vec![
///     "# comment",
///     "  error: timeout  ",
///     "",
///     "info: ok",
/// ];
/// assert_eq!(process_log_lines(&log), vec!["ERROR: TIMEOUT", "INFO: OK"]);
/// ```
pub fn process_log_lines(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(|line| line.trim())
        .map(|line| line.to_uppercase())
        .collect()
}

/// 商品リストから価格が閾値以上の商品名だけを取り出し、昇順ソートして返す。
///
/// # Examples
///
/// ```
/// use iterators::{Product, expensive_products_sorted};
/// let products = vec![
///     Product { name: "apple".to_string(), price: 200 },
///     Product { name: "melon".to_string(), price: 2000 },
///     Product { name: "grape".to_string(), price: 1500 },
/// ];
/// assert_eq!(expensive_products_sorted(&products, 1000), vec!["grape", "melon"]);
/// ```
pub fn expensive_products_sorted<'a>(products: &'a [Product], threshold: u32) -> Vec<&'a str> {
    let mut names: Vec<&str> = products
        .iter()
        .filter(|p| p.price >= threshold)
        .map(|p| p.name.as_str())
        .collect();
    names.sort_unstable();
    names
}

/// 商品を表すデータ構造。
#[derive(Debug, Clone)]
pub struct Product {
    /// 商品名
    pub name: String,
    /// 価格（円）
    pub price: u32,
}

// ============================================================================
// 6. カスタムイテレータ: Counter
// ============================================================================

/// 1 から `max` まで 1 ずつカウントアップするイテレータ。
///
/// `Iterator` トレイトを自前実装することで、標準ライブラリの
/// `map`, `filter`, `sum` などがすべて自動的に使えるようになる。
///
/// # Examples
///
/// ```
/// use iterators::Counter;
///
/// let v: Vec<u32> = Counter::new(5).collect();
/// assert_eq!(v, vec![1, 2, 3, 4, 5]);
///
/// let sum: u32 = Counter::new(5).sum();
/// assert_eq!(sum, 15);
///
/// // 2つの Counter を zip して積を求める
/// let products: Vec<u32> = Counter::new(5)
///     .zip(Counter::new(5).skip(1))
///     .map(|(a, b)| a * b)
///     .collect();
/// assert_eq!(products, vec![2, 6, 12, 20]);
/// ```
#[derive(Debug)]
pub struct Counter {
    current: u32,
    max: u32,
}

impl Counter {
    /// 1 から `max` までカウントする `Counter` を作成する。
    pub fn new(max: u32) -> Self {
        Counter { current: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;

    /// 次のカウント値を返す。`max` を超えると `None` を返す。
    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.max {
            self.current += 1;
            Some(self.current)
        } else {
            None
        }
    }
}

// ============================================================================
// 7. scan / take_while / skip_while
// ============================================================================

/// `scan` を使って累積和（ランニングサム）を計算する。
///
/// `fold` と異なり、各ステップの中間値をイテレータとして出力する。
///
/// # Examples
///
/// ```
/// use iterators::running_sum;
/// assert_eq!(running_sum(&[1, 2, 3, 4, 5]), vec![1, 3, 6, 10, 15]);
/// ```
pub fn running_sum(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .scan(0, |acc, &x| {
            *acc += x;
            Some(*acc)
        })
        .collect()
}

/// `take_while` を使って閾値以下の要素を先頭から取り出す。
///
/// 条件が偽になった時点でイテレーションを終了する。
/// その後の要素が条件を満たしていても返さない点に注意。
///
/// # Examples
///
/// ```
/// use iterators::take_while_le;
/// assert_eq!(take_while_le(&[1, 2, 3, 2, 5], 3), vec![1, 2, 3, 2]);
/// assert_eq!(take_while_le(&[5, 4, 3], 3), vec![]);
/// ```
pub fn take_while_le(numbers: &[i32], threshold: i32) -> Vec<i32> {
    numbers
        .iter()
        .copied()
        .take_while(|&x| x <= threshold)
        .collect()
}

/// `take_while` を使って閾値未満の要素を先頭から取り出す。
///
/// # Examples
///
/// ```
/// use iterators::take_below_threshold;
/// assert_eq!(take_below_threshold(&[1, 3, 5, 7, 2], 6), vec![1, 3, 5]);
/// ```
pub fn take_below_threshold(numbers: &[i32], threshold: i32) -> Vec<i32> {
    numbers
        .iter()
        .copied()
        .take_while(|&x| x < threshold)
        .collect()
}

/// `skip_while` を使って、条件が真の間の要素をスキップする。
///
/// 条件が初めて偽になったあとの要素をすべて返す。
///
/// # Examples
///
/// ```
/// use iterators::skip_leading_zeros;
/// assert_eq!(skip_leading_zeros(&[0, 0, 1, 2, 0, 3]), vec![1, 2, 0, 3]);
/// ```
pub fn skip_leading_zeros(numbers: &[i32]) -> Vec<i32> {
    numbers
        .iter()
        .copied()
        .skip_while(|&x| x == 0)
        .collect()
}

/// `scan` を使って移動平均を計算する。
///
/// `window` 個の要素が揃ってから値を出力し始める。
/// 先頭の `window - 1` 個の要素は出力されない。
///
/// # Examples
///
/// ```
/// use iterators::moving_average;
/// let result = moving_average(&[1.0, 2.0, 3.0, 4.0, 5.0], 3);
/// assert_eq!(result, vec![2.0, 3.0, 4.0]);
/// ```
pub fn moving_average(data: &[f64], window: usize) -> Vec<f64> {
    use std::collections::VecDeque;

    data.iter()
        .scan(VecDeque::new(), move |buf, &x| {
            buf.push_back(x);
            if buf.len() > window {
                buf.pop_front();
            }
            if buf.len() == window {
                Some(Some(buf.iter().sum::<f64>() / window as f64))
            } else {
                Some(None) // window が揃うまでは None を出力
            }
        })
        .flatten() // Some(None) → skip、Some(Some(v)) → Some(v)
        .collect()
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- map / filter / fold ---

    #[test]
    fn test_square_all() {
        assert_eq!(square_all(&[1, 2, 3, 4, 5]), vec![1, 4, 9, 16, 25]);
        assert_eq!(square_all(&[]), vec![]);
        assert_eq!(square_all(&[-3, 0, 3]), vec![9, 0, 9]);
    }

    #[test]
    fn test_filter_evens() {
        assert_eq!(filter_evens(&[1, 2, 3, 4, 5, 6]), vec![2, 4, 6]);
        assert_eq!(filter_evens(&[1, 3, 5]), vec![]);
        assert_eq!(filter_evens(&[2, 4, 6]), vec![2, 4, 6]);
    }

    #[test]
    fn test_sum_with_fold() {
        assert_eq!(sum_with_fold(&[1, 2, 3, 4, 5]), 15);
        assert_eq!(sum_with_fold(&[]), 0);
        assert_eq!(sum_with_fold(&[-1, 1]), 0);
    }

    #[test]
    fn test_join_with_fold() {
        assert_eq!(join_with_fold(&["Hello", ", ", "world", "!"]), "Hello, world!");
        assert_eq!(join_with_fold(&[]), "");
    }

    // --- flat_map ---

    #[test]
    fn test_words_from_sentences() {
        let sentences = vec!["hello world", "foo bar baz"];
        assert_eq!(
            words_from_sentences(&sentences),
            vec!["hello", "world", "foo", "bar", "baz"]
        );
    }

    #[test]
    fn test_flatten_nested() {
        let nested = vec![vec![1, 2, 3], vec![4, 5], vec![6]];
        assert_eq!(flatten_nested(nested), vec![1, 2, 3, 4, 5, 6]);

        let empty: Vec<Vec<i32>> = vec![vec![], vec![], vec![]];
        assert_eq!(flatten_nested(empty), vec![]);
    }

    // --- zip / chain ---

    #[test]
    fn test_zip_names_scores() {
        let result = zip_names_scores(&["Alice", "Bob", "Carol"], &[95, 87, 72]);
        assert_eq!(
            result,
            vec![
                ("Alice".to_string(), 95),
                ("Bob".to_string(), 87),
                ("Carol".to_string(), 72),
            ]
        );
    }

    #[test]
    fn test_zip_truncates_to_shorter() {
        // 短い方で打ち切られることを確認
        let result = zip_names_scores(&["Alice", "Bob", "Carol"], &[95, 87]);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_chain_slices() {
        assert_eq!(chain_slices(&[1, 2, 3], &[4, 5, 6]), vec![1, 2, 3, 4, 5, 6]);
        assert_eq!(chain_slices(&[], &[1, 2]), vec![1, 2]);
        assert_eq!(chain_slices(&[1, 2], &[]), vec![1, 2]);
    }

    // --- 無限イテレータ ---

    #[test]
    fn test_fibonacci_first_ten() {
        let result: Vec<u64> = fibonacci().take(10).collect();
        assert_eq!(result, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn test_fibonacci_under_100() {
        let result = fibonacci_under_100();
        assert_eq!(result, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]);
        assert!(result.iter().all(|&x| x < 100));
    }

    #[test]
    fn test_demonstrate_infinite_iterators() {
        let (repeated, cycled) = demonstrate_infinite_iterators();
        assert_eq!(repeated, vec![42, 42, 42]);
        assert_eq!(cycled, vec![1, 2, 3, 1, 2, 3, 1]);
    }

    // --- パイプライン ---

    #[test]
    fn test_process_log_lines() {
        let log = vec![
            "# コメント行",
            "  error: timeout  ",
            "",
            "warning: retrying",
            "# 別のコメント",
            "info: success",
        ];
        assert_eq!(
            process_log_lines(&log),
            vec!["ERROR: TIMEOUT", "WARNING: RETRYING", "INFO: SUCCESS"]
        );
    }

    #[test]
    fn test_expensive_products_sorted() {
        let products = vec![
            Product { name: "apple".to_string(), price: 200 },
            Product { name: "melon".to_string(), price: 2000 },
            Product { name: "grape".to_string(), price: 1500 },
            Product { name: "banana".to_string(), price: 500 },
        ];
        assert_eq!(
            expensive_products_sorted(&products, 1000),
            vec!["grape", "melon"]
        );
        assert_eq!(expensive_products_sorted(&products, 3000), Vec::<&str>::new());
    }

    // --- Counter カスタムイテレータ ---

    #[test]
    fn test_counter_basic() {
        let v: Vec<u32> = Counter::new(5).collect();
        assert_eq!(v, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_counter_sum() {
        let sum: u32 = Counter::new(5).sum();
        assert_eq!(sum, 15); // 1+2+3+4+5
    }

    #[test]
    fn test_counter_with_map_and_filter() {
        // 1..=5 のうち偶数を2乗する
        let result: Vec<u32> = Counter::new(5)
            .filter(|x| x % 2 == 0)
            .map(|x| x * x)
            .collect();
        assert_eq!(result, vec![4, 16]); // 2^2, 4^2
    }

    #[test]
    fn test_counter_zip_with_itself() {
        // Counter(5) と Counter(5).skip(1) を zip して積を求める
        let products: Vec<u32> = Counter::new(5)
            .zip(Counter::new(5).skip(1))
            .map(|(a, b)| a * b)
            .collect();
        // (1,2), (2,3), (3,4), (4,5) → [2, 6, 12, 20]
        assert_eq!(products, vec![2, 6, 12, 20]);
    }

    #[test]
    fn test_counter_empty() {
        let v: Vec<u32> = Counter::new(0).collect();
        assert_eq!(v, vec![]);
    }

    // --- scan / take_while / skip_while ---

    #[test]
    fn test_running_sum() {
        assert_eq!(running_sum(&[1, 2, 3, 4, 5]), vec![1, 3, 6, 10, 15]);
        assert_eq!(running_sum(&[10, -3, 5]), vec![10, 7, 12]);
        assert_eq!(running_sum(&[]), vec![]);
    }

    #[test]
    fn test_take_while_le() {
        // 閾値以下が続く間だけ取り出す
        assert_eq!(take_while_le(&[1, 2, 3, 2, 5], 3), vec![1, 2, 3, 2]);
        // 先頭要素がすでに閾値を超えている場合は空
        assert_eq!(take_while_le(&[5, 4, 3], 3), vec![]);
        // すべての要素が閾値以下の場合はそのまま
        assert_eq!(take_while_le(&[1, 2, 3], 10), vec![1, 2, 3]);
    }

    #[test]
    fn test_take_below_threshold() {
        assert_eq!(take_below_threshold(&[1, 3, 5, 7, 2], 6), vec![1, 3, 5]);
        assert_eq!(take_below_threshold(&[10, 20, 30], 5), vec![]);
        assert_eq!(take_below_threshold(&[1, 2, 3], 100), vec![1, 2, 3]);
    }

    #[test]
    fn test_skip_leading_zeros() {
        assert_eq!(skip_leading_zeros(&[0, 0, 1, 2, 0, 3]), vec![1, 2, 0, 3]);
        assert_eq!(skip_leading_zeros(&[0, 0, 0]), vec![]);
        assert_eq!(skip_leading_zeros(&[1, 2, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn test_moving_average() {
        let result = moving_average(&[1.0, 2.0, 3.0, 4.0, 5.0], 3);
        assert_eq!(result, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_moving_average_window_equals_length() {
        let result = moving_average(&[1.0, 2.0, 3.0], 3);
        assert_eq!(result, vec![2.0]);
    }

    #[test]
    fn test_moving_average_window_larger_than_data() {
        let result = moving_average(&[1.0, 2.0], 3);
        assert_eq!(result, vec![]);
    }
}

// ============================================================
// 強化: rayon による並列イテレータ
// ============================================================

use rayon::prelude::*;

/// rayon の par_iter() を使った並列 map
pub fn parallel_map(data: &[i64]) -> Vec<i64> {
    data.par_iter().map(|&x| x * x).collect()
}

/// rayon の par_iter() を使った並列 filter + collect
pub fn parallel_filter_positive(data: &[i64]) -> Vec<i64> {
    data.par_iter().filter(|&&x| x > 0).copied().collect()
}

/// rayon の par_iter().sum() による並列合計
pub fn parallel_sum(data: &[i64]) -> i64 {
    data.par_iter().sum()
}

/// rayon の par_iter() を使ったチェーン（map + filter + sum）
pub fn parallel_pipeline(data: &[i64], threshold: i64) -> i64 {
    data.par_iter()
        .map(|&x| x * 2)
        .filter(|&x| x > threshold)
        .sum()
}

#[cfg(test)]
mod rayon_tests {
    use super::*;

    #[test]
    fn test_parallel_map() {
        let data = vec![1i64, 2, 3, 4, 5];
        let mut result = parallel_map(&data);
        result.sort(); // 並列なので順序が変わる可能性がある
        assert_eq!(result, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    fn test_parallel_filter_positive() {
        let data = vec![-3i64, -1, 0, 2, 4, 6];
        let mut result = parallel_filter_positive(&data);
        result.sort();
        assert_eq!(result, vec![2, 4, 6]);
    }

    #[test]
    fn test_parallel_sum() {
        let data: Vec<i64> = (1..=100).collect();
        assert_eq!(parallel_sum(&data), 5050);
    }

    #[test]
    fn test_parallel_pipeline() {
        let data = vec![1i64, 2, 3, 4, 5];
        // map(x*2): [2, 4, 6, 8, 10]
        // filter(>5): [6, 8, 10]
        // sum: 24
        let result = parallel_pipeline(&data, 5);
        assert_eq!(result, 24);
    }

    #[test]
    fn test_parallel_vs_sequential_consistency() {
        let data: Vec<i64> = (1..=1000).collect();
        let sequential: i64 = data.iter().map(|&x| x * x).sum();
        let parallel: i64 = data.par_iter().map(|&x| x * x).sum();
        assert_eq!(sequential, parallel);
    }
}
