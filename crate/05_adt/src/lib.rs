//! 第5章: パターンマッチングと代数的データ型
//!
//! このクレートでは以下を実装しています。
//! - [`Shape`]: 直和型の例。`match` による面積計算
//! - [`List`]: 再帰的な連結リスト（Cons/Nil パターン）
//! - [`Tree`]: 二分探索木の enum 定義と操作
//! - ネストした enum のパターン分解例
//! - `if let` / `while let` の実用例
//! - ガード条件付き `match` の例

use std::f64::consts::PI;

// ============================================================
// Shape — 直和型と match による面積計算
// ============================================================

/// 図形を表す直和型。
///
/// 各バリアントは互いに排他的であり、ある時点では
/// Circle、Rectangle、Triangle のいずれかひとつの値を持ちます。
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// 円。フィールドは半径（radius）
    Circle(f64),
    /// 長方形。フィールドは (幅, 高さ)
    Rectangle(f64, f64),
    /// 三角形。フィールドは3辺の長さ (a, b, c)
    Triangle(f64, f64, f64),
}

impl Shape {
    /// 面積を返す。
    ///
    /// - `Circle(r)`: π r²
    /// - `Rectangle(w, h)`: w × h
    /// - `Triangle(a, b, c)`: ヘロンの公式
    ///
    /// # Examples
    ///
    /// ```
    /// use adt::Shape;
    /// let c = Shape::Circle(1.0);
    /// assert!((c.area() - std::f64::consts::PI).abs() < 1e-10);
    /// ```
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle(r) => PI * r * r,
            Shape::Rectangle(w, h) => w * h,
            Shape::Triangle(a, b, c) => {
                // ヘロンの公式: s = (a+b+c)/2, 面積 = sqrt(s(s-a)(s-b)(s-c))
                let s = (a + b + c) / 2.0;
                (s * (s - a) * (s - b) * (s - c)).sqrt()
            }
        }
    }

    /// 周囲長を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use adt::Shape;
    /// let r = Shape::Rectangle(3.0, 4.0);
    /// assert_eq!(r.perimeter(), 14.0);
    /// ```
    pub fn perimeter(&self) -> f64 {
        match self {
            Shape::Circle(r) => 2.0 * PI * r,
            Shape::Rectangle(w, h) => 2.0 * (w + h),
            Shape::Triangle(a, b, c) => a + b + c,
        }
    }

    /// 最も面積の大きい図形をスライスから返す。
    ///
    /// スライスが空の場合は `None` を返します。
    pub fn largest(shapes: &[Shape]) -> Option<&Shape> {
        shapes.iter().max_by(|a, b| {
            a.area()
                .partial_cmp(&b.area())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }
}

// ============================================================
// ガード条件付き match — 独立した関数として提供
// ============================================================

/// 数値を分類する文字列を返す。
///
/// `match` のガード条件（`if`）を使って範囲ごとに分類します。
///
/// # Examples
///
/// ```
/// use adt::classify_number;
/// assert_eq!(classify_number(0),   "ゼロ");
/// assert_eq!(classify_number(5),   "小さな正数");
/// assert_eq!(classify_number(-3),  "負数");
/// assert_eq!(classify_number(100), "大きな数");
/// ```
pub fn classify_number(n: i32) -> &'static str {
    match n {
        0 => "ゼロ",
        n if n > 0 && n <= 9  => "小さな正数",
        n if n > 9 && n <= 99 => "2桁の正数",
        n if n < 0            => "負数",
        _                     => "大きな数",
    }
}

// ============================================================
// List<T> — 再帰的な連結リスト（Cons/Nil パターン）
// ============================================================

/// 関数型スタイルの連結リスト。
///
/// - `Cons(head, tail)`: 先頭要素 `head` と残りのリスト `tail` を持つノード
/// - `Nil`: 空リストを表す番兵（sentinel）
///
/// スタック上に直接再帰できないため、`tail` は `Box<List<T>>` でヒープを使います。
#[derive(Debug, Clone)]
pub enum List<T> {
    /// 先頭要素と残りのリストを持つノード
    Cons(T, Box<List<T>>),
    /// 空リスト
    Nil,
}

impl<T: Clone + std::fmt::Debug> List<T> {
    /// 空のリストを生成する。
    pub fn empty() -> Self {
        List::Nil
    }

    /// 先頭に `value` を追加した新しいリストを返す（所有権を消費する）。
    ///
    /// # Examples
    ///
    /// ```
    /// use adt::List;
    /// let list = List::empty().prepend(3).prepend(2).prepend(1);
    /// assert_eq!(list.len(), 3);
    /// ```
    pub fn prepend(self, value: T) -> Self {
        List::Cons(value, Box::new(self))
    }

    /// リストの長さを返す（末尾再帰ではない再帰実装）。
    pub fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(_, tail) => 1 + tail.len(),
        }
    }

    /// リストが空かどうかを返す。
    pub fn is_empty(&self) -> bool {
        matches!(self, List::Nil)
    }

    /// リストを `Vec<T>` に変換して返す。
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        let mut current = self;
        loop {
            match current {
                List::Nil => break,
                List::Cons(head, tail) => {
                    result.push(head.clone());
                    current = tail;
                }
            }
        }
        result
    }

    /// 各要素に関数 `f` を適用した新しいリストを返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use adt::List;
    /// let list = List::empty().prepend(3).prepend(2).prepend(1);
    /// let doubled = list.map(|x| x * 2);
    /// assert_eq!(doubled.to_vec(), vec![2, 4, 6]);
    /// ```
    pub fn map<U: Clone + std::fmt::Debug, F: Fn(T) -> U>(self, f: F) -> List<U> {
        match self {
            List::Nil => List::Nil,
            List::Cons(head, tail) => List::Cons(f(head), Box::new(tail.map(f))),
        }
    }
}

impl List<i32> {
    /// 数値リストの合計を返す。
    ///
    /// 空リストに対しては `0` を返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use adt::List;
    /// let list = List::empty().prepend(3).prepend(2).prepend(1);
    /// assert_eq!(list.sum(), 6);
    /// ```
    pub fn sum(&self) -> i32 {
        match self {
            List::Nil => 0,
            List::Cons(head, tail) => head + tail.sum(),
        }
    }
}

// ============================================================
// Tree<T> — 二分探索木
// ============================================================

/// 二分探索木のノードを表す再帰的な enum。
///
/// - `Leaf`: 空のノード（木の末端）
/// - `Node`: 値と左右の子ツリーを持つノード
///
/// 挿入・検索は `T: Ord` を要求します。
#[derive(Debug, Clone)]
pub enum Tree<T> {
    /// 空のノード
    Leaf,
    /// 値と左右の子ツリーを持つノード
    Node {
        value: T,
        left: Box<Tree<T>>,
        right: Box<Tree<T>>,
    },
}

impl<T: Ord + Clone> Tree<T> {
    /// 空のツリーを生成する。
    pub fn empty() -> Self {
        Tree::Leaf
    }

    /// `new_value` を挿入した新しいツリーを返す（不変スタイル）。
    ///
    /// 二分探索木の不変条件（左 < ノード < 右）を維持します。
    /// すでに同じ値が存在する場合は変更なしで返します。
    ///
    /// # Examples
    ///
    /// ```
    /// use adt::Tree;
    /// let tree = Tree::empty().insert(5).insert(3).insert(7);
    /// assert!(tree.contains(&3));
    /// assert!(!tree.contains(&4));
    /// ```
    pub fn insert(self, new_value: T) -> Self {
        match self {
            Tree::Leaf => Tree::Node {
                value: new_value,
                left: Box::new(Tree::Leaf),
                right: Box::new(Tree::Leaf),
            },
            Tree::Node { value, left, right } => {
                if new_value < value {
                    Tree::Node {
                        value,
                        left: Box::new(left.insert(new_value)),
                        right,
                    }
                } else if new_value > value {
                    Tree::Node {
                        value,
                        left,
                        right: Box::new(right.insert(new_value)),
                    }
                } else {
                    // 重複は無視する
                    Tree::Node { value, left, right }
                }
            }
        }
    }

    /// `target` がツリーに含まれるかどうかを返す。
    pub fn contains(&self, target: &T) -> bool {
        match self {
            Tree::Leaf => false,
            Tree::Node { value, left, right } => {
                if target == value {
                    true
                } else if target < value {
                    left.contains(target)
                } else {
                    right.contains(target)
                }
            }
        }
    }

    /// 中順（in-order）走査でソートされた `Vec<T>` を返す。
    ///
    /// 二分探索木の不変条件が保たれていれば、常に昇順になります。
    pub fn to_sorted_vec(&self) -> Vec<T> {
        match self {
            Tree::Leaf => Vec::new(),
            Tree::Node { value, left, right } => {
                let mut result = left.to_sorted_vec();
                result.push(value.clone());
                result.extend(right.to_sorted_vec());
                result
            }
        }
    }

    /// ツリーのノード数を返す。
    pub fn size(&self) -> usize {
        match self {
            Tree::Leaf => 0,
            Tree::Node { left, right, .. } => 1 + left.size() + right.size(),
        }
    }

    /// ツリーの高さ（根から最も深い葉までの辺の数）を返す。
    pub fn height(&self) -> usize {
        match self {
            Tree::Leaf => 0,
            Tree::Node { left, right, .. } => {
                1 + left.height().max(right.height())
            }
        }
    }
}

// ============================================================
// ネストした enum のパターン分解例
// ============================================================

/// ネットワークイベントの種類を表す enum。
#[derive(Debug, Clone, PartialEq)]
pub enum NetworkEvent {
    /// 接続イベント。接続元のアドレスを保持する
    Connected(Address),
    /// データ受信イベント。送信元アドレスとペイロードを保持する
    DataReceived { from: Address, payload: Vec<u8> },
    /// 切断イベント。切断コードを保持する
    Disconnected(DisconnectCode),
}

/// ネットワークアドレスを表す enum。
#[derive(Debug, Clone, PartialEq)]
pub enum Address {
    /// IPv4 アドレス（4オクテット）
    Ipv4(u8, u8, u8, u8),
    /// IPv6 アドレス（簡略表現として文字列を使用）
    Ipv6(String),
}

/// 切断コードを表す enum。
#[derive(Debug, Clone, PartialEq)]
pub enum DisconnectCode {
    /// 正常切断
    Normal,
    /// タイムアウトによる切断
    Timeout,
    /// エラーによる切断（エラーコードを保持）
    Error(u32),
}

/// ネットワークイベントを人間が読める文字列に変換する。
///
/// ネストした enum を一度に分解するパターンを示します。
///
/// # Examples
///
/// ```
/// use adt::{NetworkEvent, Address, DisconnectCode, describe_event};
/// let event = NetworkEvent::Connected(Address::Ipv4(192, 168, 1, 1));
/// assert_eq!(describe_event(&event), "接続: 192.168.1.1");
/// ```
pub fn describe_event(event: &NetworkEvent) -> String {
    match event {
        // Address のネストまで一度に分解する
        NetworkEvent::Connected(Address::Ipv4(a, b, c, d)) => {
            format!("接続: {}.{}.{}.{}", a, b, c, d)
        }
        NetworkEvent::Connected(Address::Ipv6(addr)) => {
            format!("接続(IPv6): {}", addr)
        }

        // 構造体バリアントと内部 enum を同時に分解する
        NetworkEvent::DataReceived {
            from: Address::Ipv4(a, b, c, d),
            payload,
        } => {
            format!("データ受信: {}.{}.{}.{} から {} バイト", a, b, c, d, payload.len())
        }
        NetworkEvent::DataReceived { from: Address::Ipv6(addr), payload } => {
            format!("データ受信(IPv6): {} から {} バイト", addr, payload.len())
        }

        // 切断コードのネストを分解する
        NetworkEvent::Disconnected(DisconnectCode::Normal) => "正常切断".to_string(),
        NetworkEvent::Disconnected(DisconnectCode::Timeout) => "タイムアウト切断".to_string(),
        NetworkEvent::Disconnected(DisconnectCode::Error(code)) => {
            format!("エラー切断: コード {}", code)
        }
    }
}

// ============================================================
// if let / while let の実用例
// ============================================================

/// ログレベルを表す enum。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// ログエントリを表す構造体。
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    /// 新しいログエントリを生成する。
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
        }
    }
}

/// `if let` を使って Error レベルのログだけを抽出する。
///
/// # Examples
///
/// ```
/// use adt::{LogEntry, LogLevel, extract_errors};
/// let logs = vec![
///     LogEntry::new(LogLevel::Info,  "起動しました"),
///     LogEntry::new(LogLevel::Error, "接続失敗"),
///     LogEntry::new(LogLevel::Warn,  "タイムアウト"),
///     LogEntry::new(LogLevel::Error, "ディスク満杯"),
/// ];
/// let errors = extract_errors(&logs);
/// assert_eq!(errors.len(), 2);
/// ```
pub fn extract_errors(logs: &[LogEntry]) -> Vec<&str> {
    let mut errors = Vec::new();

    for entry in logs {
        // if let でパターンが合致した場合のみ処理する
        if let LogLevel::Error = entry.level {
            errors.push(entry.message.as_str());
        }
    }

    errors
}

/// `while let` を使ってキューから正の数だけを取り出す。
///
/// キューの先頭が正の数である間、要素を取り出して返します。
///
/// # Examples
///
/// ```
/// use adt::drain_positive;
/// use std::collections::VecDeque;
/// let mut queue = VecDeque::from(vec![3, 1, 4, -1, 5]);
/// let result = drain_positive(&mut queue);
/// assert_eq!(result, vec![3, 1, 4]);
/// // -1 以降はキューに残る
/// assert_eq!(queue.front(), Some(&-1));
/// ```
pub fn drain_positive(queue: &mut std::collections::VecDeque<i32>) -> Vec<i32> {
    let mut result = Vec::new();

    // while let でキューの先頭が正の数の間だけループする
    while let Some(&front) = queue.front() {
        if front > 0 {
            // front が正数のときだけ pop して結果に追加する
            result.push(queue.pop_front().unwrap());
        } else {
            break;
        }
    }

    result
}

// ============================================================
// #[cfg(test)] — テストモジュール
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    // --- Shape のテスト ---

    #[test]
    fn test_circle_area() {
        let circle = Shape::Circle(1.0);
        let expected = PI;
        assert!((circle.area() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_rectangle_area_and_perimeter() {
        let rect = Shape::Rectangle(4.0, 5.0);
        assert_eq!(rect.area(), 20.0);
        assert_eq!(rect.perimeter(), 18.0);
    }

    #[test]
    fn test_triangle_area_heron() {
        // 3-4-5 直角三角形: 面積 = 6
        let tri = Shape::Triangle(3.0, 4.0, 5.0);
        assert!((tri.area() - 6.0).abs() < 1e-10);
        assert_eq!(tri.perimeter(), 12.0);
    }

    #[test]
    fn test_shape_largest() {
        let shapes = vec![
            Shape::Circle(1.0),
            Shape::Rectangle(10.0, 10.0),
            Shape::Triangle(3.0, 4.0, 5.0),
        ];
        let largest = Shape::largest(&shapes).unwrap();
        // Rectangle(10, 10) の面積 100 が最大
        assert_eq!(*largest, Shape::Rectangle(10.0, 10.0));
    }

    #[test]
    fn test_largest_empty_slice_returns_none() {
        let shapes: Vec<Shape> = vec![];
        assert!(Shape::largest(&shapes).is_none());
    }

    // --- classify_number（ガード条件付き match）のテスト ---

    #[test]
    fn test_classify_number_zero() {
        assert_eq!(classify_number(0), "ゼロ");
    }

    #[test]
    fn test_classify_number_small_positive() {
        assert_eq!(classify_number(1), "小さな正数");
        assert_eq!(classify_number(9), "小さな正数");
    }

    #[test]
    fn test_classify_number_two_digits() {
        assert_eq!(classify_number(10), "2桁の正数");
        assert_eq!(classify_number(99), "2桁の正数");
    }

    #[test]
    fn test_classify_number_negative() {
        assert_eq!(classify_number(-1), "負数");
        assert_eq!(classify_number(-100), "負数");
    }

    #[test]
    fn test_classify_number_large() {
        assert_eq!(classify_number(100), "大きな数");
    }

    // --- List<T> のテスト ---

    #[test]
    fn test_list_empty() {
        let list: List<i32> = List::empty();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_list_prepend_and_len() {
        let list = List::empty().prepend(3).prepend(2).prepend(1);
        assert!(!list.is_empty());
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_list_to_vec_preserves_order() {
        let list = List::empty().prepend(3).prepend(2).prepend(1);
        // prepend で 1→2→3 の順に追加したので先頭から [1, 2, 3]
        assert_eq!(list.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_list_sum() {
        let list = List::empty().prepend(3).prepend(2).prepend(1);
        assert_eq!(list.sum(), 6);
    }

    #[test]
    fn test_list_sum_empty() {
        let list: List<i32> = List::empty();
        assert_eq!(list.sum(), 0);
    }

    #[test]
    fn test_list_map() {
        let list = List::empty().prepend(3).prepend(2).prepend(1);
        let doubled = list.map(|x| x * 2);
        assert_eq!(doubled.to_vec(), vec![2, 4, 6]);
    }

    // --- Tree<T> のテスト ---

    fn sample_tree() -> Tree<i32> {
        Tree::empty()
            .insert(5)
            .insert(3)
            .insert(7)
            .insert(1)
            .insert(4)
            .insert(6)
            .insert(8)
    }

    #[test]
    fn test_tree_contains_inserted_values() {
        let tree = sample_tree();
        for &v in &[1, 3, 4, 5, 6, 7, 8] {
            assert!(tree.contains(&v), "{} が見つかりません", v);
        }
    }

    #[test]
    fn test_tree_not_contains_missing_values() {
        let tree = sample_tree();
        for &v in &[0, 2, 9, 10] {
            assert!(!tree.contains(&v), "{} が誤って見つかりました", v);
        }
    }

    #[test]
    fn test_tree_to_sorted_vec() {
        let tree = sample_tree();
        assert_eq!(tree.to_sorted_vec(), vec![1, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_tree_size() {
        let tree = sample_tree();
        assert_eq!(tree.size(), 7);
    }

    #[test]
    fn test_tree_empty_contains_nothing() {
        let tree: Tree<i32> = Tree::empty();
        assert!(!tree.contains(&0));
        assert_eq!(tree.size(), 0);
        assert_eq!(tree.to_sorted_vec(), vec![]);
    }

    #[test]
    fn test_tree_duplicate_insert_does_not_grow() {
        let tree = Tree::empty().insert(5).insert(5).insert(5);
        assert_eq!(tree.size(), 1);
    }

    // --- ネストした enum パターン分解のテスト ---

    #[test]
    fn test_describe_event_connected_ipv4() {
        let event = NetworkEvent::Connected(Address::Ipv4(192, 168, 1, 1));
        assert_eq!(describe_event(&event), "接続: 192.168.1.1");
    }

    #[test]
    fn test_describe_event_data_received() {
        let event = NetworkEvent::DataReceived {
            from: Address::Ipv4(10, 0, 0, 1),
            payload: vec![0u8; 42],
        };
        assert_eq!(describe_event(&event), "データ受信: 10.0.0.1 から 42 バイト");
    }

    #[test]
    fn test_describe_event_disconnect_normal() {
        let event = NetworkEvent::Disconnected(DisconnectCode::Normal);
        assert_eq!(describe_event(&event), "正常切断");
    }

    #[test]
    fn test_describe_event_disconnect_error() {
        let event = NetworkEvent::Disconnected(DisconnectCode::Error(404));
        assert_eq!(describe_event(&event), "エラー切断: コード 404");
    }

    // --- if let / extract_errors のテスト ---

    #[test]
    fn test_extract_errors_returns_only_error_messages() {
        let logs = vec![
            LogEntry::new(LogLevel::Info, "起動しました"),
            LogEntry::new(LogLevel::Error, "接続失敗"),
            LogEntry::new(LogLevel::Warn, "タイムアウト"),
            LogEntry::new(LogLevel::Error, "ディスク満杯"),
        ];
        let errors = extract_errors(&logs);
        assert_eq!(errors, vec!["接続失敗", "ディスク満杯"]);
    }

    #[test]
    fn test_extract_errors_no_errors_returns_empty() {
        let logs = vec![
            LogEntry::new(LogLevel::Info, "正常動作"),
            LogEntry::new(LogLevel::Debug, "デバッグ情報"),
        ];
        let errors = extract_errors(&logs);
        assert!(errors.is_empty());
    }

    // --- while let / drain_positive のテスト ---

    #[test]
    fn test_drain_positive_stops_at_negative() {
        let mut queue = VecDeque::from(vec![3, 1, 4, -1, 5]);
        let result = drain_positive(&mut queue);
        assert_eq!(result, vec![3, 1, 4]);
        // -1 以降はキューに残る
        assert_eq!(queue.front(), Some(&-1));
    }

    #[test]
    fn test_drain_positive_all_positive() {
        let mut queue = VecDeque::from(vec![1, 2, 3]);
        let result = drain_positive(&mut queue);
        assert_eq!(result, vec![1, 2, 3]);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_drain_positive_starts_with_negative() {
        let mut queue = VecDeque::from(vec![-1, 2, 3]);
        let result = drain_positive(&mut queue);
        assert!(result.is_empty());
        // キューは変化しない
        assert_eq!(queue.front(), Some(&-1));
    }

    #[test]
    fn test_drain_positive_empty_queue() {
        let mut queue: VecDeque<i32> = VecDeque::new();
        let result = drain_positive(&mut queue);
        assert!(result.is_empty());
    }
}
