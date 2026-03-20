//! 第2章: Rustにおける不変性と所有権
//!
//! このクレートは以下のトピックを実例で示します。
//! - `let` / `let mut` のデフォルト不変性と関数型的意味
//! - 所有権による「値の変換」パターン
//! - `&T` を取る純粋関数（借用して計算、副作用なし）
//! - `const` / `static` を使った定数定義
//! - struct update syntax による不変な「更新」

// ---------------------------------------------------------------------------
// 定数定義 — const と static の使いどころ
// ---------------------------------------------------------------------------

/// 円周率（コンパイル時定数）。型注釈が必須。
pub const PI: f64 = std::f64::consts::PI;

/// デフォルトの接続タイムアウト秒数（コンパイル時定数）。
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// アプリケーション名（プログラム全体で唯一のメモリ上の場所を持つ）。
pub static APP_NAME: &str = "FP Study";

/// 最初の素数リスト（固定長配列のグローバル定数）。
pub static SMALL_PRIMES: [u32; 6] = [2, 3, 5, 7, 11, 13];

// ---------------------------------------------------------------------------
// Section 1: let と let mut の対比
// ---------------------------------------------------------------------------

/// 命令型スタイル: `let mut` を使って状態を更新しながら合計を求める。
///
/// `total` という可変バインディングが副作用の在り処を示している。
/// これは関数型スタイルとは対照的なアプローチ。
///
/// # Examples
///
/// ```
/// use immutability::sum_imperative;
/// assert_eq!(sum_imperative(&[1, 2, 3, 4, 5]), 15);
/// ```
pub fn sum_imperative(values: &[i32]) -> i32 {
    let mut total = 0; // 可変バインディング — 状態を持つ
    for &v in values {
        total += v;
    }
    total
}

/// 関数型スタイル: `let mut` を使わず `fold` で畳み込む。
///
/// 中間状態を持たず、累積値を関数の引数として受け渡すことで
/// 純粋な変換として合計を求める。
///
/// # Examples
///
/// ```
/// use immutability::sum_functional;
/// assert_eq!(sum_functional(&[1, 2, 3, 4, 5]), 15);
/// ```
pub fn sum_functional(values: &[i32]) -> i32 {
    values.iter().fold(0, |acc, &v| acc + v)
}

// ---------------------------------------------------------------------------
// Section 2: 所有権による「値の変換」
// ---------------------------------------------------------------------------

/// 文字列の先頭文字を大文字にした新しい `String` を返す。
///
/// 引数 `s` の所有権を受け取り、変換後の新しい文字列を返す。
/// 元の値は消費されるため、呼び出し元はこの関数に渡した変数を
/// それ以降使用できない（所有権の移動）。
///
/// # Examples
///
/// ```
/// use immutability::capitalize;
/// assert_eq!(capitalize(String::from("hello")), "Hello");
/// assert_eq!(capitalize(String::from("")), "");
/// ```
pub fn capitalize(s: String) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + chars.as_str(),
    }
}

/// 末尾に感嘆符を付けた新しい `String` を返す。
///
/// 所有権を受け取って変換するため、`+` 演算子でアロケーションを
/// 1回に抑えられる。
///
/// # Examples
///
/// ```
/// use immutability::add_exclamation;
/// assert_eq!(add_exclamation(String::from("Hello")), "Hello!");
/// ```
pub fn add_exclamation(s: String) -> String {
    s + "!"
}

/// 文字列をすべて大文字に変換した新しい `String` を返す。
///
/// # Examples
///
/// ```
/// use immutability::to_uppercase_owned;
/// assert_eq!(to_uppercase_owned(String::from("hello")), "HELLO");
/// ```
pub fn to_uppercase_owned(s: String) -> String {
    s.to_uppercase()
}

/// 正の値だけを2倍にして合計する変換パイプライン。
///
/// `let mut` を使わず、イテレータチェーンで表現した関数型スタイルの例。
///
/// # Examples
///
/// ```
/// use immutability::sum_positive_doubled;
/// assert_eq!(sum_positive_doubled(&[1, -2, 3, -4, 5]), 18);
/// ```
pub fn sum_positive_doubled(values: &[i32]) -> i32 {
    values
        .iter()
        .filter(|&&v| v > 0)
        .map(|&v| v * 2)
        .sum()
}

// ---------------------------------------------------------------------------
// Section 3: &T を取る純粋関数（借用して計算、副作用なし）
// ---------------------------------------------------------------------------

/// スライスの要素の合計を返す純粋関数。
///
/// `&[i32]` を借用するだけで所有権を取らない。
/// 入力を変更せず、外部状態にも依存しないため純粋関数である。
///
/// # Examples
///
/// ```
/// use immutability::sum;
/// assert_eq!(sum(&[10, 20, 30]), 60);
/// ```
pub fn sum(values: &[i32]) -> i32 {
    values.iter().sum()
}

/// スライスの最大値を返す純粋関数。空スライスの場合は `None` を返す。
///
/// # Examples
///
/// ```
/// use immutability::max_value;
/// assert_eq!(max_value(&[3, 1, 4, 1, 5, 9]), Some(9));
/// assert_eq!(max_value(&[]), None);
/// ```
pub fn max_value(values: &[i32]) -> Option<i32> {
    values.iter().copied().max()
}

/// 文字列が回文かどうかを判定する純粋関数。
///
/// `&str` を借用して判定し、元の文字列は一切変更しない。
///
/// # Examples
///
/// ```
/// use immutability::is_palindrome;
/// assert!(is_palindrome("racecar"));
/// assert!(is_palindrome("madam"));
/// assert!(!is_palindrome("hello"));
/// ```
pub fn is_palindrome(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    chars.iter().zip(chars.iter().rev()).all(|(a, b)| a == b)
}

/// 円の面積を計算する純粋関数。
///
/// `const PI` を参照するだけで外部状態に依存せず、副作用もない。
///
/// # Examples
///
/// ```
/// use immutability::circle_area;
/// let area = circle_area(1.0);
/// assert!((area - std::f64::consts::PI).abs() < 1e-10);
/// ```
pub fn circle_area(radius: f64) -> f64 {
    PI * radius * radius
}

/// スライスから60点以上の要素に5点加算した新しい `Vec` を返す純粋関数。
///
/// 元のスライスは変更せず、変換結果を新しいコレクションとして返す。
///
/// # Examples
///
/// ```
/// use immutability::process_scores;
/// let scores = vec![45, 72, 88, 55, 91, 60];
/// let result = process_scores(&scores);
/// assert_eq!(result, vec![77, 93, 96, 65]);
/// ```
pub fn process_scores(scores: &[u32]) -> Vec<u32> {
    scores
        .iter()
        .filter(|&&s| s >= 60)
        .map(|&s| s + 5)
        .collect()
}

// ---------------------------------------------------------------------------
// Section 4: 不変な struct を struct update syntax で「更新」する
// ---------------------------------------------------------------------------

/// アプリケーションの接続設定を表す不変な構造体。
///
/// フィールドを「変更」したい場合は、`with_*` メソッドで
/// 新しいインスタンスを作成する。元の値は変わらない。
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// 接続先ホスト名
    pub host: String,
    /// ポート番号
    pub port: u16,
    /// 最大同時接続数
    pub max_connections: u32,
    /// タイムアウト秒数
    pub timeout_secs: u64,
}

impl Config {
    /// デフォルト設定で `Config` を作成する。
    ///
    /// # Examples
    ///
    /// ```
    /// use immutability::Config;
    /// let config = Config::new();
    /// assert_eq!(config.host, "localhost");
    /// assert_eq!(config.port, 8080);
    /// ```
    pub fn new() -> Self {
        Config {
            host: String::from("localhost"),
            port: 8080,
            max_connections: 100,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    /// ポート番号だけを変えた新しい `Config` を返す。
    ///
    /// struct update syntax (`..self.clone()`) を使い、
    /// 変更するフィールドだけを指定する。元の `Config` は変更しない。
    ///
    /// # Examples
    ///
    /// ```
    /// use immutability::Config;
    /// let default = Config::new();
    /// let custom = default.with_port(443);
    /// assert_eq!(custom.port, 443);
    /// assert_eq!(custom.host, default.host); // 他のフィールドは変わらない
    /// ```
    pub fn with_port(&self, port: u16) -> Self {
        Config {
            port,
            ..self.clone()
        }
    }

    /// ホスト名だけを変えた新しい `Config` を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use immutability::Config;
    /// let default = Config::new();
    /// let prod = default.with_host("example.com");
    /// assert_eq!(prod.host, "example.com");
    /// assert_eq!(prod.port, default.port); // 他のフィールドは変わらない
    /// ```
    pub fn with_host(&self, host: impl Into<String>) -> Self {
        Config {
            host: host.into(),
            ..self.clone()
        }
    }

    /// 最大接続数だけを変えた新しい `Config` を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use immutability::Config;
    /// let default = Config::new();
    /// let high_load = default.with_max_connections(500);
    /// assert_eq!(high_load.max_connections, 500);
    /// ```
    pub fn with_max_connections(&self, max_connections: u32) -> Self {
        Config {
            max_connections,
            ..self.clone()
        }
    }

    /// タイムアウト秒数だけを変えた新しい `Config` を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use immutability::Config;
    /// let default = Config::new();
    /// let fast = default.with_timeout(5);
    /// assert_eq!(fast.timeout_secs, 5);
    /// ```
    pub fn with_timeout(&self, timeout_secs: u64) -> Self {
        Config {
            timeout_secs,
            ..self.clone()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Section 6: 永続データ構造（Persistent Data Structure）
// ---------------------------------------------------------------------------

use std::rc::Rc;

/// 永続連結リスト。
///
/// `Rc<T>` による参照カウントと構造共有（Structural Sharing）で、
/// 更新後も古いバージョンが O(1) のコストで保存される。
#[derive(Debug, Clone, PartialEq)]
pub enum PersistentList<T> {
    /// 空リスト
    Nil,
    /// 先頭要素 + 残りのリストへの共有参照
    Cons(T, Rc<PersistentList<T>>),
}

impl<T: Clone> PersistentList<T> {
    /// 空の永続リストを作る。
    pub fn empty() -> Rc<Self> {
        Rc::new(PersistentList::Nil)
    }

    /// 先頭に `value` を追加した新しいリストを返す。
    ///
    /// `tail` を共有するため O(1)。元のリストは変わらない。
    ///
    /// # Examples
    ///
    /// ```
    /// use immutability::PersistentList;
    /// use std::rc::Rc;
    ///
    /// let list = PersistentList::cons(1,
    ///     PersistentList::cons(2, PersistentList::empty()));
    /// assert_eq!(list.head(), Some(&1));
    /// ```
    pub fn cons(value: T, tail: Rc<Self>) -> Rc<Self> {
        Rc::new(PersistentList::Cons(value, tail))
    }

    /// 先頭要素への参照を返す。空なら `None`。
    pub fn head(&self) -> Option<&T> {
        match self {
            PersistentList::Cons(v, _) => Some(v),
            PersistentList::Nil => None,
        }
    }

    /// 先頭を除いた残りのリストへの参照を返す。空なら `None`。
    pub fn tail_ref(&self) -> Option<Rc<Self>> {
        match self {
            PersistentList::Cons(_, tail) => Some(Rc::clone(tail)),
            PersistentList::Nil => None,
        }
    }

    /// リストの長さを返す（O(n)）。
    pub fn len(&self) -> usize {
        match self {
            PersistentList::Nil => 0,
            PersistentList::Cons(_, tail) => 1 + tail.len(),
        }
    }

    /// 空かどうかを返す。
    pub fn is_empty(&self) -> bool {
        matches!(self, PersistentList::Nil)
    }

    /// リストを `Vec` に変換する（テスト用途）。
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        let mut current: &PersistentList<T> = self;
        loop {
            match current {
                PersistentList::Nil => break,
                PersistentList::Cons(v, tail) => {
                    result.push(v.clone());
                    current = tail;
                }
            }
        }
        result
    }
}

/// 永続スタックの push 操作。
///
/// 元のスタックは変えず、新しい先頭を持つスタックを返す。
pub fn stack_push<T: Clone>(stack: Rc<PersistentList<T>>, value: T) -> Rc<PersistentList<T>> {
    PersistentList::cons(value, stack)
}

/// 永続スタックの pop 操作。
///
/// 先頭要素と残りのスタックをタプルで返す。空なら `None`。
pub fn stack_pop<T: Clone>(stack: &Rc<PersistentList<T>>) -> Option<(T, Rc<PersistentList<T>>)> {
    match stack.as_ref() {
        PersistentList::Nil => None,
        PersistentList::Cons(v, tail) => Some((v.clone(), Rc::clone(tail))),
    }
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Section 1: let vs let mut ---

    #[test]
    fn test_sum_imperative_and_functional_agree() {
        let values = [1, 2, 3, 4, 5];
        assert_eq!(
            sum_imperative(&values),
            sum_functional(&values),
            "命令型と関数型の合計は一致するはず"
        );
    }

    #[test]
    fn test_sum_functional_empty() {
        assert_eq!(sum_functional(&[]), 0, "空スライスの合計は 0");
    }

    // --- Section 2: 所有権による値の変換 ---

    #[test]
    fn test_capitalize_basic() {
        assert_eq!(capitalize(String::from("hello")), "Hello");
        assert_eq!(capitalize(String::from("world")), "World");
    }

    #[test]
    fn test_capitalize_empty_string() {
        assert_eq!(capitalize(String::new()), "");
    }

    #[test]
    fn test_capitalize_already_upper() {
        assert_eq!(capitalize(String::from("Hello")), "Hello");
    }

    #[test]
    fn test_value_transformation_chain() {
        // 所有権の移動チェーン: String -> 大文字 -> 感嘆符付き
        let result = add_exclamation(to_uppercase_owned(String::from("hello")));
        assert_eq!(result, "HELLO!");
    }

    #[test]
    fn test_sum_positive_doubled() {
        assert_eq!(sum_positive_doubled(&[1, -2, 3, -4, 5]), 18);
        assert_eq!(sum_positive_doubled(&[-1, -2, -3]), 0);
        assert_eq!(sum_positive_doubled(&[]), 0);
    }

    // --- Section 3: &T を取る純粋関数 ---

    #[test]
    fn test_sum_pure_function() {
        let values = vec![10, 20, 30];
        let result = sum(&values);
        // sum は values を変更しないため、呼び出し後も values を使える
        assert_eq!(result, 60);
        assert_eq!(values, vec![10, 20, 30], "元のVecは変更されていない");
    }

    #[test]
    fn test_max_value() {
        assert_eq!(max_value(&[3, 1, 4, 1, 5, 9, 2, 6]), Some(9));
        assert_eq!(max_value(&[-5, -3, -1]), Some(-1));
        assert_eq!(max_value(&[]), None, "空スライスはNone");
    }

    #[test]
    fn test_is_palindrome() {
        assert!(is_palindrome("racecar"));
        assert!(is_palindrome("madam"));
        assert!(is_palindrome("a"));
        assert!(is_palindrome(""));
        assert!(!is_palindrome("hello"));
        assert!(!is_palindrome("rust"));
    }

    #[test]
    fn test_circle_area_uses_const_pi() {
        let area = circle_area(1.0);
        // const PI を使って計算しているため、標準ライブラリの PI と一致する
        assert!((area - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_process_scores_does_not_mutate_original() {
        let original = vec![45u32, 72, 88, 55, 91, 60];
        let processed = process_scores(&original);
        // 元のスライスは変更されていない
        assert_eq!(original, vec![45, 72, 88, 55, 91, 60]);
        assert_eq!(processed, vec![77, 93, 96, 65]);
    }

    // --- Section 4: struct update syntax ---

    #[test]
    fn test_config_with_port_does_not_mutate_original() {
        let default = Config::new();
        let custom = default.with_port(443);

        // default は変更されていない
        assert_eq!(default.port, 8080);
        // custom は新しいインスタンス
        assert_eq!(custom.port, 443);
        // 変更していないフィールドは default と同じ値
        assert_eq!(custom.host, default.host);
        assert_eq!(custom.max_connections, default.max_connections);
        assert_eq!(custom.timeout_secs, default.timeout_secs);
    }

    #[test]
    fn test_config_method_chain() {
        let config = Config::new()
            .with_host("example.com")
            .with_port(443)
            .with_max_connections(500)
            .with_timeout(10);

        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 443);
        assert_eq!(config.max_connections, 500);
        assert_eq!(config.timeout_secs, 10);
    }

    #[test]
    fn test_config_default_trait() {
        let config: Config = Default::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_struct_update_syntax_preserves_unmodified_fields() {
        let base = Config {
            host: String::from("db.internal"),
            port: 5432,
            max_connections: 50,
            timeout_secs: 60,
        };
        // ポートだけ変えた新しい Config
        let replica = Config {
            port: 5433,
            ..base.clone()
        };
        assert_eq!(replica.port, 5433);
        assert_eq!(replica.host, "db.internal"); // base から引き継がれた
        assert_eq!(replica.max_connections, 50);  // base から引き継がれた
        // base 自体は変わっていない
        assert_eq!(base.port, 5432);
    }

    // --- 定数のテスト ---

    #[test]
    fn test_constants_are_accessible() {
        assert_eq!(APP_NAME, "FP Study");
        assert_eq!(DEFAULT_TIMEOUT_SECS, 30);
        assert_eq!(SMALL_PRIMES[0], 2);
        assert_eq!(SMALL_PRIMES.len(), 6);
    }

    // --- Section 6: 永続データ構造 ---

    #[test]
    fn test_persistent_list_empty() {
        let list: Rc<PersistentList<i32>> = PersistentList::empty();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.head(), None);
    }

    #[test]
    fn test_persistent_list_cons() {
        let list = PersistentList::cons(
            1,
            PersistentList::cons(2, PersistentList::cons(3, PersistentList::empty())),
        );
        assert_eq!(list.head(), Some(&1));
        assert_eq!(list.len(), 3);
        assert_eq!(list.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_persistent_list_structural_sharing() {
        // base = [1, 2, 3]
        let base = PersistentList::cons(
            1,
            PersistentList::cons(2, PersistentList::cons(3, PersistentList::empty())),
        );

        // extended = [0, 1, 2, 3]  ← base の先頭を共有
        let extended = PersistentList::cons(0, Rc::clone(&base));

        // base は変わらない
        assert_eq!(base.to_vec(), vec![1, 2, 3]);
        assert_eq!(extended.to_vec(), vec![0, 1, 2, 3]);

        // 構造共有: extended の tail と base は同じ Rc を指す
        let extended_tail = extended.tail_ref().unwrap();
        assert!(Rc::ptr_eq(&base, &extended_tail));
    }

    #[test]
    fn test_persistent_list_multiple_versions() {
        let v0 = PersistentList::empty();
        let v1 = PersistentList::cons(10, Rc::clone(&v0));
        let v2 = PersistentList::cons(20, Rc::clone(&v1));
        let v3 = PersistentList::cons(30, Rc::clone(&v2));

        // v3 を作った後も v0, v1, v2 は保存されている
        assert_eq!(v0.to_vec(), Vec::<i32>::new());
        assert_eq!(v1.to_vec(), vec![10]);
        assert_eq!(v2.to_vec(), vec![20, 10]);
        assert_eq!(v3.to_vec(), vec![30, 20, 10]);
    }

    #[test]
    fn test_stack_push_pop() {
        let s0 = PersistentList::empty();
        let s1 = stack_push(s0, 10);
        let s2 = stack_push(Rc::clone(&s1), 20);
        let s3 = stack_push(Rc::clone(&s2), 30);

        // pop は先頭と残りを返す
        let (top, rest) = stack_pop(&s3).unwrap();
        assert_eq!(top, 30);
        assert_eq!(rest.head(), Some(&20));

        // 元のスタックは変わらない
        assert_eq!(s2.head(), Some(&20));
        assert_eq!(s1.head(), Some(&10));
    }

    #[test]
    fn test_stack_pop_empty() {
        let empty: Rc<PersistentList<i32>> = PersistentList::empty();
        assert_eq!(stack_pop(&empty), None);
    }

    #[test]
    fn test_persistent_list_immutability() {
        // 元のリストへの参照を保持しつつ、別のバージョンを作れる
        let original = PersistentList::cons(5, PersistentList::empty());
        let snapshot = Rc::clone(&original);

        let extended = PersistentList::cons(99, Rc::clone(&original));

        // original（snapshot 経由）は変わっていない
        assert_eq!(snapshot.head(), Some(&5));
        assert_eq!(extended.head(), Some(&99));
        assert_eq!(extended.tail_ref().unwrap().head(), Some(&5));
    }
}
