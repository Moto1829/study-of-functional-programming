//! # 第1章: 関数型プログラミングの基礎概念
//!
//! このクレートは「純粋関数」「参照透過性」「副作用の分離」「命令型 vs 関数型」を
//! Rust のコードで具体的に示します。

// ---------------------------------------------------------------------------
// 1. 純粋関数の例
// ---------------------------------------------------------------------------

/// 2つの整数を加算する純粋関数。
///
/// - 同じ引数には常に同じ値を返す（決定論的）
/// - 外部の状態を一切読み書きしない（副作用なし）
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}

/// 円の面積を返す純粋関数。
///
/// `std::f64::consts::PI` はコンパイル時定数であり外部状態ではない。
pub fn circle_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}

/// スライスの各要素を2倍にした新しい `Vec` を返す純粋関数。
///
/// 元のスライスは変更しない。
pub fn double_all(numbers: &[i32]) -> Vec<i32> {
    numbers.iter().map(|&n| n * 2).collect()
}

/// n の階乗を返す純粋関数。
///
/// 再帰によって実装されているが、外部状態に依存しないため純粋関数である。
pub fn factorial(n: u64) -> u64 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}

// ---------------------------------------------------------------------------
// 2. 不純な関数の例
// ---------------------------------------------------------------------------

/// 値を標準出力へ表示してからそのまま返す不純な関数。
///
/// # 不純である理由
/// `println!` は標準出力への書き込みという**副作用**を持つ。
/// 同じ引数を渡しても外部（画面）の状態が変化するため純粋ではない。
pub fn print_and_return(x: i32) -> i32 {
    println!("値: {}", x);
    x
}

/// 現在時刻（UNIX秒）を返す不純な関数。
///
/// # 不純である理由
/// 呼び出すたびに異なる値を返す可能性があり、**参照透過性**がない。
pub fn current_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("システム時刻が UNIX エポック以前です")
        .as_secs()
}

// ---------------------------------------------------------------------------
// 3. 参照透過性の例
// ---------------------------------------------------------------------------

/// 整数の2乗を返す純粋関数。
///
/// `square(5)` は常に `25` であり、その式をどこで `25` に置き換えても
/// プログラムの意味は変わらない。これが**参照透過性**である。
///
/// # 例
/// ```
/// use basics::square;
/// // 以下の2式は等価（square(5) を 25 で置き換えられる）
/// assert_eq!(square(5) + square(5), 25 + 25);
/// ```
pub fn square(x: i32) -> i32 {
    x * x
}

/// タイムスタンプを引数として受け取り、挨拶文字列を返す純粋関数。
///
/// 時刻の取得（副作用）を呼び出し側に委ねることで参照透過性を確保している。
/// 同じ `(name, timestamp)` の組には常に同じ文字列を返す。
///
/// # 例
/// ```
/// use basics::greet;
/// assert_eq!(greet("Alice", 1000), "[1000] Hello, Alice!");
/// ```
pub fn greet(name: &str, timestamp: u64) -> String {
    format!("[{}] Hello, {}!", timestamp, name)
}

// ---------------------------------------------------------------------------
// 4. 副作用の分離例
// ---------------------------------------------------------------------------

/// 文字列が回文かどうかを判定する純粋関数（コアロジック）。
///
/// Unicode コードポイント単位で比較する。
pub fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    chars[..len / 2]
        .iter()
        .zip(chars[len / 2 + len % 2..].iter().rev())
        .all(|(a, b)| a == b)
}

/// 単語リストから回文のみを抽出して返す純粋関数（コアロジック）。
///
/// I/O を持たないため単体テストが容易である。
pub fn filter_palindromes<'a>(words: &[&'a str]) -> Vec<&'a str> {
    words.iter().copied().filter(|w| is_palindrome(w)).collect()
}

/// 回文を標準出力に表示する関数（副作用あり）。
///
/// 純粋関数 [`filter_palindromes`] でコアロジックを処理し、
/// I/O を担う副作用はこの関数の中だけに閉じ込める。
pub fn print_palindromes(words: &[&str]) {
    let palindromes = filter_palindromes(words); // 純粋な計算
    for word in &palindromes {
        println!("回文: {}", word); // 副作用はここだけ
    }
}

// ---------------------------------------------------------------------------
// 5. 命令型 vs 関数型の対比
// ---------------------------------------------------------------------------

/// 偶数の合計を命令型スタイルで求める関数。
///
/// ループと可変変数を使い、「どのように（How）」処理するかを記述している。
pub fn sum_evens_imperative(numbers: &[i32]) -> i32 {
    let mut total = 0;
    for &n in numbers {
        if n % 2 == 0 {
            total += n;
        }
    }
    total
}

/// 偶数の合計を関数型スタイルで求める関数。
///
/// イテレータのメソッドチェーンで「何を（What）」求めるかを宣言的に記述している。
/// 可変変数が不要であり、意図が直接コードに現れる。
pub fn sum_evens_functional(numbers: &[i32]) -> i32 {
    numbers.iter().filter(|&&n| n % 2 == 0).sum()
}

/// 各要素の2乗の合計を命令型スタイルで求める関数。
pub fn sum_of_squares_imperative(numbers: &[i32]) -> i32 {
    let mut result = 0;
    for &n in numbers {
        result += n * n;
    }
    result
}

/// 各要素の2乗の合計を関数型スタイルで求める関数。
///
/// `map` で変換し `sum` で集約する。変換と集約の2段階が明示的に分かれている。
pub fn sum_of_squares_functional(numbers: &[i32]) -> i32 {
    numbers.iter().map(|&n| n * n).sum()
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- 純粋関数のテスト ---

    #[test]
    fn test_add_pure() {
        // 同じ引数には常に同じ値を返す
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(2, 3), 5); // 何度呼んでも同じ
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_circle_area_pure() {
        let area = circle_area(1.0);
        assert!((area - std::f64::consts::PI).abs() < 1e-10);
        assert!((circle_area(0.0)).abs() < 1e-10);
    }

    #[test]
    fn test_double_all_pure() {
        let original = vec![1, 2, 3];
        let doubled = double_all(&original);
        assert_eq!(doubled, vec![2, 4, 6]);
        // 元のベクタは変更されていない
        assert_eq!(original, vec![1, 2, 3]);
    }

    #[test]
    fn test_factorial_pure() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3_628_800);
    }

    // --- 参照透過性のテスト ---

    #[test]
    fn test_square_referential_transparency() {
        // square(5) を 25 に置き換えても結果は変わらない
        let via_function = square(5) + square(5);
        let via_value = 25 + 25;
        assert_eq!(via_function, via_value);
    }

    #[test]
    fn test_greet_referential_transparency() {
        // 同じ引数には常に同じ文字列を返す
        assert_eq!(greet("Alice", 1000), "[1000] Hello, Alice!");
        assert_eq!(greet("Alice", 1000), "[1000] Hello, Alice!");
        assert_eq!(greet("Bob", 2000), "[2000] Hello, Bob!");
    }

    // --- 副作用の分離テスト ---

    #[test]
    fn test_is_palindrome_pure() {
        assert!(is_palindrome("level"));
        assert!(is_palindrome("radar"));
        assert!(is_palindrome("civic"));
        assert!(is_palindrome("a"));
        assert!(is_palindrome(""));
        assert!(!is_palindrome("hello"));
        assert!(!is_palindrome("world"));
    }

    #[test]
    fn test_filter_palindromes_pure() {
        let words = vec!["level", "hello", "radar", "world", "civic"];
        let result = filter_palindromes(&words);
        assert_eq!(result, vec!["level", "radar", "civic"]);
    }

    #[test]
    fn test_filter_palindromes_empty() {
        let words: Vec<&str> = vec![];
        assert_eq!(filter_palindromes(&words), Vec::<&str>::new());
    }

    // --- 命令型 vs 関数型の等価性テスト ---

    #[test]
    fn test_sum_evens_equivalence() {
        let numbers: Vec<i32> = (1..=10).collect();
        assert_eq!(
            sum_evens_imperative(&numbers),
            sum_evens_functional(&numbers)
        );
        assert_eq!(sum_evens_functional(&numbers), 30);
    }

    #[test]
    fn test_sum_of_squares_equivalence() {
        let numbers = vec![1, 2, 3, 4, 5];
        assert_eq!(
            sum_of_squares_imperative(&numbers),
            sum_of_squares_functional(&numbers)
        );
        // 1^2 + 2^2 + 3^2 + 4^2 + 5^2 = 1 + 4 + 9 + 16 + 25 = 55
        assert_eq!(sum_of_squares_functional(&numbers), 55);
    }

    #[test]
    fn test_sum_evens_empty() {
        let empty: Vec<i32> = vec![];
        assert_eq!(sum_evens_imperative(&empty), 0);
        assert_eq!(sum_evens_functional(&empty), 0);
    }

    #[test]
    fn test_sum_of_squares_single_element() {
        let numbers = vec![7];
        assert_eq!(sum_of_squares_imperative(&numbers), 49);
        assert_eq!(sum_of_squares_functional(&numbers), 49);
    }
}
