/// イテレータ（Iterators）の例
///
/// Rustのイテレータは遅延評価され、必要になるまで計算しません。
/// `map`、`filter`、`flat_map`、`take`、`zip` などのアダプタを
/// チェーンして宣言的なデータ変換パイプラインを構築できます。

/// カスタムイテレータ: フィボナッチ数列を生成する
pub struct Fibonacci {
    a: u64,
    b: u64,
}

impl Fibonacci {
    pub fn new() -> Self {
        Fibonacci { a: 0, b: 1 }
    }
}

impl Default for Fibonacci {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let next = self.a + self.b;
        self.a = self.b;
        self.b = next;
        Some(self.a)
    }
}

/// イテレータを使って偶数の二乗の合計を求める
pub fn sum_of_squared_evens(numbers: &[i32]) -> i32 {
    numbers
        .iter()
        .filter(|&&n| n % 2 == 0)
        .map(|&n| n * n)
        .sum()
}

/// イテレータを使って文字列のリストを加工する
pub fn process_words(words: &[&str]) -> Vec<String> {
    words
        .iter()
        .filter(|w| w.len() > 3)
        .map(|w| w.to_uppercase())
        .collect()
}

/// zip で2つのイテレータを組み合わせる
pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// flat_map でネストされた構造を平坦化する
pub fn flatten_and_double(nested: &[Vec<i32>]) -> Vec<i32> {
    nested
        .iter()
        .flat_map(|inner| inner.iter().map(|&n| n * 2))
        .collect()
}

fn main() {
    println!("=== イテレータ（Iterators）===");

    // 基本的なイテレータ操作
    let numbers: Vec<i32> = (1..=10).collect();
    println!("numbers = {:?}", numbers);

    let sum_sq_evens = sum_of_squared_evens(&numbers);
    println!("sum of squared evens (1..=10) = {}", sum_sq_evens);

    // 文字列の処理
    let words = vec!["hi", "rust", "functional", "fp", "programming"];
    let processed = process_words(&words);
    println!("processed words = {:?}", processed);

    // カスタムイテレータ: フィボナッチ
    let fibs: Vec<u64> = Fibonacci::new().take(10).collect();
    println!("fibonacci (first 10) = {:?}", fibs);

    // zip でドット積
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![4.0, 5.0, 6.0];
    println!("dot_product({:?}, {:?}) = {}", a, b, dot_product(&a, &b));

    // flat_map で平坦化
    let nested = vec![vec![1, 2], vec![3, 4], vec![5]];
    let flat = flatten_and_double(&nested);
    println!("flatten_and_double = {:?}", flat);

    // 遅延評価: 無限イテレータから最初の5つの偶数フィボナッチ数を取得
    let even_fibs: Vec<u64> = Fibonacci::new()
        .filter(|n| n % 2 == 0)
        .take(5)
        .collect();
    println!("even fibonacci (first 5) = {:?}", even_fibs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_of_squared_evens() {
        let nums = vec![1, 2, 3, 4, 5];
        // 2^2 + 4^2 = 4 + 16 = 20
        assert_eq!(sum_of_squared_evens(&nums), 20);
    }

    #[test]
    fn test_process_words() {
        let words = vec!["hi", "rust", "fp", "code"];
        let result = process_words(&words);
        assert_eq!(result, vec!["RUST", "CODE"]);
    }

    #[test]
    fn test_dot_product() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert!((dot_product(&a, &b) - 32.0).abs() < 1e-10);
    }

    #[test]
    fn test_flatten_and_double() {
        let nested = vec![vec![1, 2], vec![3, 4]];
        assert_eq!(flatten_and_double(&nested), vec![2, 4, 6, 8]);
    }

    #[test]
    fn test_fibonacci_iterator() {
        let fibs: Vec<u64> = Fibonacci::new().take(8).collect();
        assert_eq!(fibs, vec![1, 1, 2, 3, 5, 8, 13, 21]);
    }

    #[test]
    fn test_even_fibonacci() {
        let even_fibs: Vec<u64> = Fibonacci::new()
            .filter(|n| n % 2 == 0)
            .take(4)
            .collect();
        assert_eq!(even_fibs, vec![2, 8, 34, 144]);
    }
}
