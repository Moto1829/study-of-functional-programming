//! # 第6章: Option型とResult型による関数型エラーハンドリング
//!
//! このクレートは以下のトピックを Rust のコードで具体的に示します。
//!
//! - `Option<T>` を返す関数と `map` / `and_then` チェーン
//! - `Result<T, E>` を返す関数と `map` / `and_then` チェーン
//! - `?` 演算子を使った複数エラーの連鎖処理
//! - カスタムエラー型 `AppError` と `Display` 実装
//! - 複数エラー型を統合する `From` トレイト実装
//! - `Option` と `Result` の相互変換
//! - イテレータと `Result` を組み合わせた処理

use std::collections::HashMap;
use std::fmt;
use std::num::ParseIntError;

// ---------------------------------------------------------------------------
// 1. Option を返す関数群と map / and_then チェーン
// ---------------------------------------------------------------------------

/// 文字列スライスを `u32` にパースする。変換できなければ `None` を返す。
///
/// # Examples
///
/// ```
/// use error_handling::parse_u32;
/// assert_eq!(parse_u32("42"), Some(42));
/// assert_eq!(parse_u32("-1"), None);
/// assert_eq!(parse_u32("abc"), None);
/// ```
pub fn parse_u32(s: &str) -> Option<u32> {
    s.parse::<u32>().ok()
}

/// `HashMap<&str, u32>` からキーで検索する。見つからなければ `None`。
///
/// # Examples
///
/// ```
/// use error_handling::lookup;
/// use std::collections::HashMap;
/// let mut map = HashMap::new();
/// map.insert("age", 30u32);
/// assert_eq!(lookup(&map, "age"), Some(30));
/// assert_eq!(lookup(&map, "missing"), None);
/// ```
pub fn lookup(map: &HashMap<&str, u32>, key: &str) -> Option<u32> {
    map.get(key).copied()
}

/// 値が指定範囲内（`min` 以上 `max` 以下）であれば `Some(n)`、そうでなければ `None`。
///
/// # Examples
///
/// ```
/// use error_handling::check_range;
/// assert_eq!(check_range(50, 0, 100), Some(50));
/// assert_eq!(check_range(200, 0, 100), None);
/// ```
pub fn check_range(n: u32, min: u32, max: u32) -> Option<u32> {
    if n >= min && n <= max {
        Some(n)
    } else {
        None
    }
}

/// 文字列を受け取り、`u32` パース → 範囲チェック（1–99）→ 2 乗 の
/// `map` / `and_then` / `filter` チェーンを示す例。
///
/// - パースできない → `None`
/// - 0 または 100 以上 → `None`
/// - それ以外 → `Some(n * n)`
///
/// # Examples
///
/// ```
/// use error_handling::option_chain;
/// assert_eq!(option_chain("9"), Some(81));
/// assert_eq!(option_chain("0"), None);
/// assert_eq!(option_chain("abc"), None);
/// ```
pub fn option_chain(s: &str) -> Option<u32> {
    parse_u32(s.trim())
        .filter(|&n| n > 0 && n < 100) // 1–99 のみ通過
        .map(|n| n * n)                 // 2 乗
}

/// スライスの先頭要素を `u32` にパースし、2 桁（10–99）であれば 2 倍にして返す。
///
/// `and_then` で Option を返す処理を 2 段階連鎖する例。
///
/// # Examples
///
/// ```
/// use error_handling::find_double_digit;
/// assert_eq!(find_double_digit(&["42", "ignored"]), Some(84));
/// assert_eq!(find_double_digit(&["9"]), None);
/// assert_eq!(find_double_digit(&[]), None);
/// ```
pub fn find_double_digit(items: &[&str]) -> Option<u32> {
    items
        .first()                                 // Option<&&str>
        .and_then(|s| parse_u32(s))              // Option<u32>: パース
        .filter(|&n| n >= 10 && n <= 99)         // 2 桁のみ通過
        .map(|n| n * 2)                          // 2 倍
}

// ---------------------------------------------------------------------------
// 2. Result を返す関数群と map / and_then チェーン
// ---------------------------------------------------------------------------

/// 文字列を `i32` にパースする。失敗時は `ParseIntError` を返す。
///
/// # Examples
///
/// ```
/// use error_handling::parse_i32;
/// assert!(parse_i32("42").is_ok());
/// assert!(parse_i32("abc").is_err());
/// ```
pub fn parse_i32(s: &str) -> Result<i32, ParseIntError> {
    s.trim().parse::<i32>()
}

/// 正の整数であれば `Ok(n as u32)`、そうでなければ `Err` を返す。
///
/// # Examples
///
/// ```
/// use error_handling::ensure_positive;
/// assert_eq!(ensure_positive(5), Ok(5u32));
/// assert!(ensure_positive(-1).is_err());
/// assert!(ensure_positive(0).is_err());
/// ```
pub fn ensure_positive(n: i32) -> Result<u32, String> {
    if n > 0 {
        Ok(n as u32)
    } else {
        Err(format!("{} は正の数ではありません", n))
    }
}

/// `map` で成功値を変換し、`map_err` でエラー値を変換する例。
///
/// "10" → Ok(20)、"abc" → Err("パースエラー: ...")
///
/// # Examples
///
/// ```
/// use error_handling::result_map_chain;
/// assert_eq!(result_map_chain("10"), Ok(20));
/// assert!(result_map_chain("abc").is_err());
/// ```
pub fn result_map_chain(s: &str) -> Result<i32, String> {
    parse_i32(s)
        .map(|n| n * 2)                            // Ok の値を 2 倍
        .map_err(|e| format!("パースエラー: {}", e)) // Err のメッセージを整形
}

/// `and_then` で Result を返す処理を 2 段チェーンする例。
///
/// 文字列 → i32 パース → 正数チェック の順に処理する。
///
/// # Examples
///
/// ```
/// use error_handling::result_and_then_chain;
/// assert_eq!(result_and_then_chain("42"), Ok(42u32));
/// assert!(result_and_then_chain("-1").is_err());
/// assert!(result_and_then_chain("abc").is_err());
/// ```
pub fn result_and_then_chain(s: &str) -> Result<u32, String> {
    parse_i32(s)
        .map_err(|e| e.to_string())   // ParseIntError → String（エラー型を統一）
        .and_then(ensure_positive)    // i32 → u32（正数チェック付き）
}

// ---------------------------------------------------------------------------
// 3. ? 演算子を使った関数（複数のエラーが起きうる処理を連鎖）
// ---------------------------------------------------------------------------

/// アプリケーション全体で使うカスタムエラー型。
///
/// 複数の異なるエラー種別を 1 つの enum にまとめる。
#[derive(Debug, PartialEq)]
pub enum AppError {
    /// 文字列パースに失敗した
    ParseError(String),
    /// 値が許容範囲外
    OutOfRange { value: i32, min: i32, max: i32 },
    /// 0 による除算
    DivisionByZero,
    /// 設定キーが見つからない
    KeyNotFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ParseError(msg) => {
                write!(f, "パースエラー: {}", msg)
            }
            AppError::OutOfRange { value, min, max } => {
                write!(f, "値 {} は範囲 [{}, {}] の外です", value, min, max)
            }
            AppError::DivisionByZero => {
                write!(f, "0 による除算は許可されていません")
            }
            AppError::KeyNotFound(key) => {
                write!(f, "キー '{}' が見つかりません", key)
            }
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // ParseError は元のエラー型を保持していないためここでは None
        None
    }
}

// ---------------------------------------------------------------------------
// 4. 複数のエラー型を統合する From トレイトの実装
// ---------------------------------------------------------------------------

/// `ParseIntError` を `AppError::ParseError` に自動変換する。
///
/// これにより `parse::<i32>()?` を `Result<_, AppError>` を返す関数内で
/// そのまま使える。
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::ParseError(e.to_string())
    }
}

/// 文字列を受け取り、`AppError` を使って範囲チェックまで行う。
///
/// `?` 演算子が `From<ParseIntError>` を介して自動でエラー変換を行う。
///
/// # Examples
///
/// ```
/// use error_handling::{parse_and_validate, AppError};
/// assert_eq!(parse_and_validate("42", 0, 100), Ok(42));
/// assert!(matches!(parse_and_validate("abc", 0, 100), Err(AppError::ParseError(_))));
/// assert!(matches!(parse_and_validate("200", 0, 100), Err(AppError::OutOfRange { .. })));
/// ```
pub fn parse_and_validate(s: &str, min: i32, max: i32) -> Result<i32, AppError> {
    let n: i32 = s.trim().parse()?; // ParseIntError → AppError::ParseError（From 自動適用）

    if n < min || n > max {
        return Err(AppError::OutOfRange { value: n, min, max });
    }

    Ok(n)
}

/// 2 つの文字列をパースし、安全に除算する。
///
/// `?` を複数箇所で使い、異なる種類のエラーを連鎖させる例。
///
/// # Errors
///
/// - 文字列のパースに失敗した場合: `AppError::ParseError`
/// - 除数が 0 の場合: `AppError::DivisionByZero`
///
/// # Examples
///
/// ```
/// use error_handling::{safe_divide, AppError};
/// assert_eq!(safe_divide("10", "2"), Ok(5));
/// assert_eq!(safe_divide("10", "0"), Err(AppError::DivisionByZero));
/// assert!(matches!(safe_divide("abc", "2"), Err(AppError::ParseError(_))));
/// ```
pub fn safe_divide(a: &str, b: &str) -> Result<i32, AppError> {
    let dividend: i32 = a.trim().parse()?; // ? で ParseIntError → AppError
    let divisor: i32 = b.trim().parse()?;  // ? で ParseIntError → AppError

    if divisor == 0 {
        return Err(AppError::DivisionByZero);
    }

    Ok(dividend / divisor)
}

/// 設定マップからキーを取得し、値をパースして範囲チェックを行う。
///
/// `Option` を `ok_or_else` で `Result` に変換してから `?` を使う例。
///
/// # Errors
///
/// - キーが存在しない場合: `AppError::KeyNotFound`
/// - パースに失敗した場合: `AppError::ParseError`
/// - 範囲外の場合: `AppError::OutOfRange`
///
/// # Examples
///
/// ```
/// use error_handling::{fetch_and_parse_config, AppError};
/// use std::collections::HashMap;
///
/// let mut config = HashMap::new();
/// config.insert("port", "8080");
/// assert_eq!(fetch_and_parse_config(&config, "port", 1, 65535), Ok(8080));
/// assert!(matches!(
///     fetch_and_parse_config(&config, "missing", 1, 65535),
///     Err(AppError::KeyNotFound(_))
/// ));
/// ```
pub fn fetch_and_parse_config(
    config: &HashMap<&str, &str>,
    key: &str,
    min: i32,
    max: i32,
) -> Result<i32, AppError> {
    // Option → Result への変換: None を KeyNotFound エラーに
    let raw = config
        .get(key)
        .ok_or_else(|| AppError::KeyNotFound(key.to_string()))?;

    parse_and_validate(raw, min, max) // ParseError / OutOfRange を伝搬
}

// ---------------------------------------------------------------------------
// 5. Option と Result の相互変換例（ok(), ok_or(), transpose()）
// ---------------------------------------------------------------------------

/// `Option` と `Result` の相互変換パターンをまとめた関数。
///
/// 戻り値は各変換結果を `String` にまとめたものを返す（デモ用）。
pub fn conversion_demo() -> Vec<String> {
    let mut results = Vec::new();

    // --- Option → Result ---
    let opt_some: Option<i32> = Some(42);
    let res: Result<i32, &str> = opt_some.ok_or("値がありませんでした");
    results.push(format!("Some(42).ok_or(...) = {:?}", res)); // Ok(42)

    let opt_none: Option<i32> = None;
    let res2: Result<i32, &str> = opt_none.ok_or("値がありませんでした");
    results.push(format!("None.ok_or(...) = {:?}", res2)); // Err("...")

    // ok_or_else: None のときだけクロージャを呼ぶ（遅延評価）
    let res3: Result<i32, String> = opt_none.ok_or_else(|| "遅延エラー生成".to_string());
    results.push(format!("None.ok_or_else(...) = {:?}", res3));

    // --- Result → Option ---
    let ok_val: Result<i32, &str> = Ok(42);
    let opt: Option<i32> = ok_val.ok();
    results.push(format!("Ok(42).ok() = {:?}", opt)); // Some(42)

    let err_val: Result<i32, &str> = Err("失敗");
    let opt2: Option<i32> = err_val.ok();
    results.push(format!("Err(...).ok() = {:?}", opt2)); // None

    // err(): Ok を None に、Err を Some に変換
    let ok_val2: Result<i32, &str> = Ok(42);
    let opt_err: Option<&str> = ok_val2.err();
    results.push(format!("Ok(42).err() = {:?}", opt_err)); // None

    // --- transpose ---
    // Option<Result<T, E>> → Result<Option<T>, E>
    let some_ok: Option<Result<i32, &str>> = Some(Ok(42));
    let transposed: Result<Option<i32>, &str> = some_ok.transpose();
    results.push(format!("Some(Ok(42)).transpose() = {:?}", transposed)); // Ok(Some(42))

    let some_err: Option<Result<i32, &str>> = Some(Err("失敗"));
    let transposed2: Result<Option<i32>, &str> = some_err.transpose();
    results.push(format!("Some(Err(...)).transpose() = {:?}", transposed2)); // Err("失敗")

    let none_val: Option<Result<i32, &str>> = None;
    let transposed3: Result<Option<i32>, &str> = none_val.transpose();
    results.push(format!("None.transpose() = {:?}", transposed3)); // Ok(None)

    results
}

// ---------------------------------------------------------------------------
// 6. イテレータと Result の処理
// ---------------------------------------------------------------------------

/// 文字列スライスをすべて `i32` にパースして `Vec` に集める。
///
/// 1 つでもパースに失敗した場合は `Err` を返す（全件成功が条件）。
/// `collect::<Result<Vec<_>, _>>()` の活用例。
///
/// # Examples
///
/// ```
/// use error_handling::parse_all;
/// assert_eq!(parse_all(&["1", "2", "3"]), Ok(vec![1, 2, 3]));
/// assert!(parse_all(&["1", "abc", "3"]).is_err());
/// ```
pub fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, ParseIntError> {
    inputs
        .iter()
        .map(|s| s.trim().parse::<i32>()) // Iterator<Item = Result<i32, ParseIntError>>
        .collect() // 全部 Ok なら Ok(Vec<i32>)、1 つでも Err なら最初の Err
}

/// 文字列スライスのうちパースできたものだけを `Vec` に集める。
///
/// パース失敗は無視する（エラーを `None` として除外）。
/// `filter_map` + `Result::ok` の活用例。
///
/// # Examples
///
/// ```
/// use error_handling::parse_valid_only;
/// assert_eq!(parse_valid_only(&["1", "abc", "3", "xyz", "5"]), vec![1, 3, 5]);
/// assert_eq!(parse_valid_only(&["a", "b"]), Vec::<i32>::new());
/// ```
pub fn parse_valid_only(inputs: &[&str]) -> Vec<i32> {
    inputs
        .iter()
        .filter_map(|s| s.trim().parse::<i32>().ok()) // Err を None として除外
        .collect()
}

/// 文字列スライスをすべてパースし、成功すれば合計を返す。
///
/// 1 つでもパース失敗があれば `Err` を返す。
///
/// # Examples
///
/// ```
/// use error_handling::sum_strings;
/// assert_eq!(sum_strings(&["1", "2", "3"]), Ok(6));
/// assert!(sum_strings(&["1", "abc"]).is_err());
/// ```
pub fn sum_strings(inputs: &[&str]) -> Result<i32, ParseIntError> {
    let numbers: Result<Vec<i32>, _> = inputs
        .iter()
        .map(|s| s.trim().parse::<i32>())
        .collect();

    numbers.map(|v| v.iter().sum())
}

/// パース成功したものだけの合計を返す。失敗は無視する。
///
/// # Examples
///
/// ```
/// use error_handling::sum_valid_strings;
/// assert_eq!(sum_valid_strings(&["1", "abc", "3", "xyz", "5"]), 9);
/// assert_eq!(sum_valid_strings(&["a", "b"]), 0);
/// ```
pub fn sum_valid_strings(inputs: &[&str]) -> i32 {
    parse_valid_only(inputs).iter().sum()
}

/// 文字列スライスの各要素をパースして `AppError` を使って検証し、
/// すべて成功した場合のみ `Vec` を返す。
///
/// `map` + `collect::<Result<_, _>>()` に加え、カスタムエラー型と
/// 組み合わせた例。
///
/// # Examples
///
/// ```
/// use error_handling::{validate_all, AppError};
/// assert_eq!(validate_all(&["1", "50", "99"], 0, 100), Ok(vec![1, 50, 99]));
/// assert!(matches!(
///     validate_all(&["1", "200", "3"], 0, 100),
///     Err(AppError::OutOfRange { .. })
/// ));
/// ```
pub fn validate_all(inputs: &[&str], min: i32, max: i32) -> Result<Vec<i32>, AppError> {
    inputs
        .iter()
        .map(|s| parse_and_validate(s, min, max))
        .collect() // collect は Result<Vec<_>, _> として動作
}

// ---------------------------------------------------------------------------
// テストモジュール
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Option チェーン ---

    #[test]
    fn test_option_chain_valid() {
        assert_eq!(option_chain("9"), Some(81));
        assert_eq!(option_chain("  12  "), Some(144)); // 空白込み
    }

    #[test]
    fn test_option_chain_zero_is_filtered() {
        // 0 は filter で除外される
        assert_eq!(option_chain("0"), None);
    }

    #[test]
    fn test_option_chain_out_of_range() {
        // 100 以上は除外
        assert_eq!(option_chain("100"), None);
        assert_eq!(option_chain("999"), None);
    }

    #[test]
    fn test_option_chain_non_numeric() {
        assert_eq!(option_chain("abc"), None);
        assert_eq!(option_chain(""), None);
    }

    #[test]
    fn test_find_double_digit_valid() {
        assert_eq!(find_double_digit(&["42", "ignored"]), Some(84));
        assert_eq!(find_double_digit(&["10"]), Some(20));
        assert_eq!(find_double_digit(&["99"]), Some(198));
    }

    #[test]
    fn test_find_double_digit_single_digit() {
        // 1 桁は 2 桁チェックで除外
        assert_eq!(find_double_digit(&["9"]), None);
        assert_eq!(find_double_digit(&["0"]), None);
    }

    #[test]
    fn test_find_double_digit_empty_slice() {
        assert_eq!(find_double_digit(&[]), None);
    }

    // --- Result チェーン ---

    #[test]
    fn test_result_map_chain_ok() {
        assert_eq!(result_map_chain("10"), Ok(20));
        assert_eq!(result_map_chain("-5"), Ok(-10));
    }

    #[test]
    fn test_result_map_chain_err() {
        let r = result_map_chain("xyz");
        assert!(r.is_err());
        assert!(r.unwrap_err().contains("パースエラー"));
    }

    #[test]
    fn test_result_and_then_chain_ok() {
        assert_eq!(result_and_then_chain("42"), Ok(42u32));
    }

    #[test]
    fn test_result_and_then_chain_negative() {
        let r = result_and_then_chain("-1");
        assert!(r.is_err());
    }

    #[test]
    fn test_result_and_then_chain_parse_err() {
        let r = result_and_then_chain("not_a_number");
        assert!(r.is_err());
    }

    // --- カスタムエラー型と ? 演算子 ---

    #[test]
    fn test_parse_and_validate_ok() {
        assert_eq!(parse_and_validate("42", 0, 100), Ok(42));
    }

    #[test]
    fn test_parse_and_validate_out_of_range() {
        let r = parse_and_validate("200", 0, 100);
        assert_eq!(
            r,
            Err(AppError::OutOfRange {
                value: 200,
                min: 0,
                max: 100
            })
        );
    }

    #[test]
    fn test_parse_and_validate_parse_error() {
        let r = parse_and_validate("abc", 0, 100);
        assert!(matches!(r, Err(AppError::ParseError(_))));
    }

    #[test]
    fn test_safe_divide_ok() {
        assert_eq!(safe_divide("10", "2"), Ok(5));
    }

    #[test]
    fn test_safe_divide_by_zero() {
        assert_eq!(safe_divide("10", "0"), Err(AppError::DivisionByZero));
    }

    #[test]
    fn test_safe_divide_parse_error() {
        let r = safe_divide("abc", "2");
        assert!(matches!(r, Err(AppError::ParseError(_))));
    }

    #[test]
    fn test_fetch_config_ok() {
        let mut config = HashMap::new();
        config.insert("port", "8080");
        assert_eq!(fetch_and_parse_config(&config, "port", 1, 65535), Ok(8080));
    }

    #[test]
    fn test_fetch_config_key_not_found() {
        let config: HashMap<&str, &str> = HashMap::new();
        let r = fetch_and_parse_config(&config, "missing", 1, 65535);
        assert!(matches!(r, Err(AppError::KeyNotFound(_))));
    }

    #[test]
    fn test_fetch_config_out_of_range() {
        let mut config = HashMap::new();
        config.insert("port", "99999");
        let r = fetch_and_parse_config(&config, "port", 1, 65535);
        assert!(matches!(r, Err(AppError::OutOfRange { .. })));
    }

    // --- AppError の Display 実装 ---

    #[test]
    fn test_app_error_display_division_by_zero() {
        let msg = AppError::DivisionByZero.to_string();
        assert!(msg.contains("0 による除算"));
    }

    #[test]
    fn test_app_error_display_out_of_range() {
        let msg = AppError::OutOfRange {
            value: 200,
            min: 0,
            max: 100,
        }
        .to_string();
        assert!(msg.contains("200"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_app_error_display_key_not_found() {
        let msg = AppError::KeyNotFound("timeout".to_string()).to_string();
        assert!(msg.contains("timeout"));
    }

    // --- Option と Result の相互変換 ---

    #[test]
    fn test_ok_or_some() {
        let opt: Option<i32> = Some(42);
        assert_eq!(opt.ok_or("err"), Ok(42));
    }

    #[test]
    fn test_ok_or_none() {
        let opt: Option<i32> = None;
        assert_eq!(opt.ok_or("err"), Err("err"));
    }

    #[test]
    fn test_result_ok_converts_ok() {
        let r: Result<i32, &str> = Ok(42);
        assert_eq!(r.ok(), Some(42));
    }

    #[test]
    fn test_result_ok_converts_err_to_none() {
        let r: Result<i32, &str> = Err("fail");
        assert_eq!(r.ok(), None);
    }

    #[test]
    fn test_transpose_some_ok() {
        let v: Option<Result<i32, &str>> = Some(Ok(42));
        assert_eq!(v.transpose(), Ok(Some(42)));
    }

    #[test]
    fn test_transpose_some_err() {
        let v: Option<Result<i32, &str>> = Some(Err("fail"));
        assert_eq!(v.transpose(), Err("fail"));
    }

    #[test]
    fn test_transpose_none() {
        let v: Option<Result<i32, &str>> = None;
        assert_eq!(v.transpose(), Ok(None));
    }

    // --- イテレータと Result ---

    #[test]
    fn test_parse_all_ok() {
        assert_eq!(parse_all(&["1", "2", "3"]), Ok(vec![1, 2, 3]));
    }

    #[test]
    fn test_parse_all_with_invalid() {
        assert!(parse_all(&["1", "abc", "3"]).is_err());
    }

    #[test]
    fn test_parse_all_empty() {
        assert_eq!(parse_all(&[]), Ok(vec![]));
    }

    #[test]
    fn test_parse_valid_only() {
        assert_eq!(
            parse_valid_only(&["1", "abc", "3", "xyz", "5"]),
            vec![1, 3, 5]
        );
    }

    #[test]
    fn test_parse_valid_only_all_invalid() {
        assert_eq!(parse_valid_only(&["a", "b", "c"]), Vec::<i32>::new());
    }

    #[test]
    fn test_sum_strings_ok() {
        assert_eq!(sum_strings(&["1", "2", "3"]), Ok(6));
    }

    #[test]
    fn test_sum_strings_err() {
        assert!(sum_strings(&["1", "abc"]).is_err());
    }

    #[test]
    fn test_sum_valid_strings() {
        assert_eq!(sum_valid_strings(&["1", "abc", "3", "xyz", "5"]), 9);
        assert_eq!(sum_valid_strings(&["a", "b"]), 0);
    }

    #[test]
    fn test_validate_all_ok() {
        assert_eq!(validate_all(&["1", "50", "99"], 0, 100), Ok(vec![1, 50, 99]));
    }

    #[test]
    fn test_validate_all_out_of_range() {
        let r = validate_all(&["1", "200", "3"], 0, 100);
        assert!(matches!(r, Err(AppError::OutOfRange { .. })));
    }

    #[test]
    fn test_validate_all_parse_error() {
        let r = validate_all(&["1", "xyz", "3"], 0, 100);
        assert!(matches!(r, Err(AppError::ParseError(_))));
    }
}

// ============================================================
// 強化: thiserror と anyhow の実用的な使い方
// ============================================================

// ─── thiserror によるドメインエラー定義 ──────────────────────

use thiserror::Error;

/// thiserror を使ったドメインエラー
#[derive(Debug, Error)]
pub enum UserError {
    #[error("ユーザーが見つかりません: id={0}")]
    NotFound(u32),

    #[error("メールアドレスの形式が不正です: {email}")]
    InvalidEmail { email: String },

    #[error("パスワードが短すぎます: {len}文字 (最低{min}文字必要)")]
    PasswordTooShort { len: usize, min: usize },
}

/// thiserror を使ったインフラエラー
#[derive(Debug, Error)]
pub enum InfraError {
    #[error("データベースエラー: {0}")]
    Database(String),

    #[error("ネットワークエラー: {0}")]
    Network(String),
}

/// thiserror でエラーのラップ（#[from] 属性）
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("ユーザーエラー: {0}")]
    User(#[from] UserError),

    #[error("インフラエラー: {0}")]
    Infra(#[from] InfraError),
}

/// thiserror を使ったバリデーション関数
pub fn validate_email(email: &str) -> Result<(), UserError> {
    if email.contains('@') {
        Ok(())
    } else {
        Err(UserError::InvalidEmail { email: email.to_string() })
    }
}

pub fn validate_password(password: &str) -> Result<(), UserError> {
    const MIN_LEN: usize = 8;
    if password.len() >= MIN_LEN {
        Ok(())
    } else {
        Err(UserError::PasswordTooShort { len: password.len(), min: MIN_LEN })
    }
}

// ─── anyhow によるアプリケーション層エラー処理 ───────────────

use anyhow::{Context, Result as AnyResult, bail, ensure};

/// anyhow を使ったシンプルなエラー処理
pub fn parse_port(s: &str) -> AnyResult<u16> {
    let port: u16 = s.parse().context("ポート番号は数値で指定してください")?;
    ensure!(port > 0, "ポート番号は1以上である必要があります");
    Ok(port)
}

/// anyhow の bail! マクロ
pub fn check_age(age: i32) -> AnyResult<()> {
    if age < 0 {
        bail!("年齢は0以上でなければなりません: {}", age);
    }
    if age > 150 {
        bail!("現実的な年齢を入力してください: {}", age);
    }
    Ok(())
}

/// anyhow と thiserror の組み合わせ
/// ドメイン層: thiserror でエラーを定義
/// アプリケーション層: anyhow でエラーを伝播
pub fn create_user(email: &str, password: &str) -> AnyResult<String> {
    validate_email(email)
        .context("メールアドレスのバリデーションに失敗しました")?;
    validate_password(password)
        .context("パスワードのバリデーションに失敗しました")?;

    Ok(format!("ユーザー作成成功: {}", email))
}

#[cfg(test)]
mod thiserror_anyhow_tests {
    use super::*;

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        let err = validate_email("not-an-email").unwrap_err();
        assert!(matches!(err, UserError::InvalidEmail { .. }));
        assert!(err.to_string().contains("not-an-email"));
    }

    #[test]
    fn test_validate_password_short() {
        let err = validate_password("abc").unwrap_err();
        assert!(matches!(err, UserError::PasswordTooShort { len: 3, min: 8 }));
    }

    #[test]
    fn test_parse_port_valid() {
        assert_eq!(parse_port("8080").unwrap(), 8080);
    }

    #[test]
    fn test_parse_port_invalid_format() {
        let err = parse_port("abc").unwrap_err();
        assert!(err.to_string().contains("ポート番号は数値"));
    }

    #[test]
    fn test_check_age_negative() {
        assert!(check_age(-1).is_err());
    }

    #[test]
    fn test_check_age_valid() {
        assert!(check_age(25).is_ok());
    }

    #[test]
    fn test_create_user_success() {
        let result = create_user("alice@example.com", "securepass").unwrap();
        assert!(result.contains("alice@example.com"));
    }

    #[test]
    fn test_create_user_invalid_email() {
        let err = create_user("not-email", "securepass").unwrap_err();
        assert!(err.to_string().contains("メールアドレス"));
    }

    #[test]
    fn test_service_error_from_user_error() {
        let user_err = UserError::NotFound(42);
        let svc_err: ServiceError = user_err.into();
        assert!(matches!(svc_err, ServiceError::User(_)));
    }
}
