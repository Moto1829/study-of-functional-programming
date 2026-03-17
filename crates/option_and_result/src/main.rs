/// Option と Result 型の例
///
/// `Option<T>` と `Result<T, E>` はRustにおけるモナド的な型です。
/// null参照やエラーを型安全に扱うための仕組みを提供します。
/// `map`、`and_then`、`unwrap_or` などでチェーンして使います。

use std::num::ParseIntError;

/// Option を使った安全な除算
pub fn safe_divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

/// Option のチェーン: 文字列から偶数を取得する
pub fn parse_even(s: &str) -> Option<i32> {
    s.trim()
        .parse::<i32>()
        .ok()
        .filter(|&n| n % 2 == 0)
}

/// Result を使ったエラーハンドリング
pub fn parse_positive(s: &str) -> Result<u32, String> {
    let n: i32 = s.trim().parse().map_err(|_| format!("'{}' は整数ではありません", s))?;
    if n < 0 {
        Err(format!("{} は負の値です", n))
    } else {
        Ok(n as u32)
    }
}

/// Result のチェーン: 文字列をパースして計算する
pub fn parse_and_add(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x: i32 = a.trim().parse()?;
    let y: i32 = b.trim().parse()?;
    Ok(x + y)
}

/// Option と Result の変換
pub fn first_positive(numbers: &[i32]) -> Option<i32> {
    numbers.iter().find(|&&n| n > 0).copied()
}

/// map, and_then を使った変換チェーン
pub fn process_input(input: &str) -> Option<String> {
    input
        .trim()
        .parse::<i32>()
        .ok()
        .filter(|&n| n > 0)
        .map(|n| n * 2)
        .map(|n| format!("結果: {}", n))
}

fn main() {
    println!("=== Option と Result ===");

    // Option の使用例
    println!("safe_divide(10.0, 2.0) = {:?}", safe_divide(10.0, 2.0));
    println!("safe_divide(10.0, 0.0) = {:?}", safe_divide(10.0, 0.0));

    // Option のチェーン
    println!("parse_even(\"4\")  = {:?}", parse_even("4"));
    println!("parse_even(\"3\")  = {:?}", parse_even("3"));
    println!("parse_even(\"abc\")= {:?}", parse_even("abc"));

    // unwrap_or でデフォルト値を使用
    let result = safe_divide(10.0, 0.0).unwrap_or(f64::INFINITY);
    println!("safe_divide(10.0, 0.0).unwrap_or(∞) = {}", result);

    // Result の使用例
    println!("parse_positive(\"42\")  = {:?}", parse_positive("42"));
    println!("parse_positive(\"-5\")  = {:?}", parse_positive("-5"));
    println!("parse_positive(\"abc\") = {:?}", parse_positive("abc"));

    // Result のチェーン
    println!("parse_and_add(\"3\", \"4\")  = {:?}", parse_and_add("3", "4"));
    println!("parse_and_add(\"3\", \"x\")  = {:?}", parse_and_add("3", "x"));

    // Option から値を取り出す
    let numbers = vec![-3, -1, 0, 5, 8];
    match first_positive(&numbers) {
        Some(n) => println!("最初の正の数: {}", n),
        None => println!("正の数はありません"),
    }

    // 変換チェーン
    println!("process_input(\"5\")  = {:?}", process_input("5"));
    println!("process_input(\"-1\") = {:?}", process_input("-1"));
    println!("process_input(\"abc\")= {:?}", process_input("abc"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_divide_success() {
        assert_eq!(safe_divide(10.0, 2.0), Some(5.0));
    }

    #[test]
    fn test_safe_divide_by_zero() {
        assert_eq!(safe_divide(10.0, 0.0), None);
    }

    #[test]
    fn test_parse_even_success() {
        assert_eq!(parse_even("4"), Some(4));
        assert_eq!(parse_even("  8  "), Some(8));
    }

    #[test]
    fn test_parse_even_odd() {
        assert_eq!(parse_even("3"), None);
    }

    #[test]
    fn test_parse_even_invalid() {
        assert_eq!(parse_even("abc"), None);
    }

    #[test]
    fn test_parse_positive_success() {
        assert_eq!(parse_positive("42"), Ok(42));
        assert_eq!(parse_positive("0"), Ok(0));
    }

    #[test]
    fn test_parse_positive_negative() {
        assert!(parse_positive("-5").is_err());
    }

    #[test]
    fn test_parse_positive_invalid() {
        assert!(parse_positive("abc").is_err());
    }

    #[test]
    fn test_parse_and_add_success() {
        assert_eq!(parse_and_add("3", "4"), Ok(7));
    }

    #[test]
    fn test_parse_and_add_failure() {
        assert!(parse_and_add("3", "x").is_err());
    }

    #[test]
    fn test_first_positive() {
        assert_eq!(first_positive(&[-1, -2, 3, 4]), Some(3));
        assert_eq!(first_positive(&[-1, -2, -3]), None);
    }

    #[test]
    fn test_process_input() {
        assert_eq!(process_input("5"), Some("結果: 10".to_string()));
        assert_eq!(process_input("-1"), None);
        assert_eq!(process_input("abc"), None);
    }
}
