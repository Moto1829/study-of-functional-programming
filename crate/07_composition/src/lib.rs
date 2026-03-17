// ============================================================
// 第7章: 関数合成とコンビネータパターン
// ============================================================

// ============================================================
// 1. 関数合成: compose / pipe
// ============================================================

/// 2つの関数を合成する。数学の `g ∘ f` に対応する。
///
/// `compose(f, g)` は `|x| g(f(x))` を返す。
/// 引数の順は「先に適用する関数が左」であることに注意。
///
/// # 例
///
/// ```
/// use composition::compose;
///
/// let add_one = |x: i32| x + 1;
/// let double = |x: i32| x * 2;
/// let add_then_double = compose(add_one, double);
/// assert_eq!(add_then_double(3), 8); // (3+1)*2
/// ```
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

/// データが左から右に流れるように2つの関数を合成する。
///
/// `pipe(f, g)` は `|x| g(f(x))` を返す。
/// `compose` と処理は同一だが、「データが f → g の順に流れる」という
/// 読みやすさを意図した命名になっている。
///
/// # 例
///
/// ```
/// use composition::pipe;
///
/// let trim = |s: &str| s.trim().to_string();
/// let to_upper = |s: String| s.to_uppercase();
/// let normalize = pipe(trim, to_upper);
/// assert_eq!(normalize("  hello  "), "HELLO");
/// ```
pub fn pipe<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

/// 3つの関数を左から右に合成する。
///
/// `pipe3(f, g, h)` は `|x| h(g(f(x)))` を返す。
///
/// # 例
///
/// ```
/// use composition::pipe3;
///
/// let f = |x: i32| x + 1;
/// let g = |x: i32| x * 2;
/// let h = |x: i32| x - 3;
/// let pipeline = pipe3(f, g, h);
/// assert_eq!(pipeline(3), 5); // (3+1)*2-3
/// ```
pub fn pipe3<A, B, C, D, F, G, H>(f: F, g: G, h: H) -> impl Fn(A) -> D
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
    H: Fn(C) -> D,
{
    move |x| h(g(f(x)))
}

/// 複数の関数を左から右に合成するマクロ。
///
/// `pipe_all!(f, g, h, ...)` は `|x| ...(h(g(f(x))))` と等価。
/// 2つ以上の任意個数の関数に対応する。
///
/// # 例
///
/// ```
/// use composition::pipe_all;
///
/// let f = |x: i32| x + 1;
/// let g = |x: i32| x * 2;
/// let h = |x: i32| x - 3;
/// let i = |x: i32| x.to_string();
/// let pipeline = pipe_all!(f, g, h, i);
/// assert_eq!(pipeline(3), "5");
/// ```
#[macro_export]
macro_rules! pipe_all {
    ($f:expr) => { $f };
    ($f:expr, $($rest:expr),+) => {
        {
            let f = $f;
            let rest = $crate::pipe_all!($($rest),+);
            move |x| rest(f(x))
        }
    };
}

// ============================================================
// 2. コンビネータを使った Builder パターン
// ============================================================

/// SQLのSELECTクエリを段階的に構築するビルダー。
///
/// 各メソッドは `self` を消費して変更後の `self` を返すコンビネータとして設計されており、
/// メソッドチェーンによる宣言的なクエリ構築ができる。
///
/// # 例
///
/// ```
/// use composition::QueryBuilder;
///
/// let sql = QueryBuilder::new("users")
///     .select("id")
///     .select("name")
///     .where_clause("age > 18")
///     .order_by("name")
///     .limit(10)
///     .build();
///
/// assert_eq!(
///     sql,
///     "SELECT id, name FROM users WHERE age > 18 ORDER BY name LIMIT 10"
/// );
/// ```
#[derive(Debug, Default)]
pub struct QueryBuilder {
    table: String,
    columns: Vec<String>,
    conditions: Vec<String>,
    limit: Option<usize>,
    order_by: Option<String>,
}

impl QueryBuilder {
    /// 対象テーブルを指定して新しい `QueryBuilder` を生成する。
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            ..Default::default()
        }
    }

    /// SELECTするカラムを追加する。
    ///
    /// 呼び出しのたびにカラムが蓄積される。
    /// 一度も呼ばれなかった場合は `SELECT *` になる。
    #[must_use]
    pub fn select(mut self, column: impl Into<String>) -> Self {
        self.columns.push(column.into());
        self
    }

    /// WHERE条件を追加する。
    ///
    /// 複数回呼ぶと `AND` で結合される。
    #[must_use]
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.conditions.push(condition.into());
        self
    }

    /// 取得件数の上限（LIMIT）を設定する。
    #[must_use]
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// ORDER BYカラムを設定する。
    #[must_use]
    pub fn order_by(mut self, column: impl Into<String>) -> Self {
        self.order_by = Some(column.into());
        self
    }

    /// 設定をもとにSQL文字列を生成する。
    ///
    /// このメソッドを呼ぶと `self` が消費され、`QueryBuilder` は使えなくなる。
    pub fn build(self) -> String {
        let columns = if self.columns.is_empty() {
            "*".to_string()
        } else {
            self.columns.join(", ")
        };

        let mut sql = format!("SELECT {} FROM {}", columns, self.table);

        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.conditions.join(" AND "));
        }

        if let Some(order) = self.order_by {
            sql.push_str(&format!(" ORDER BY {}", order));
        }

        if let Some(n) = self.limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }

        sql
    }
}

// ============================================================
// 3. 簡単なパーサーコンビネータ
// ============================================================

/// パーサーの結果型。
///
/// 成功時は `(残りの入力文字列, 解析した値)` のタプル、
/// 失敗時はエラーメッセージの文字列を返す。
pub type ParseResult<'a, T> = Result<(&'a str, T), String>;

/// 特定の文字1つにマッチする基本パーサーを生成する。
///
/// # 例
///
/// ```
/// use composition::char_parser;
///
/// let parse_a = char_parser('a');
/// assert_eq!(parse_a("abc"), Ok(("bc", 'a')));
/// assert!(parse_a("xyz").is_err());
/// ```
pub fn char_parser(expected: char) -> impl Fn(&str) -> ParseResult<char> {
    move |input: &str| {
        let mut chars = input.chars();
        match chars.next() {
            Some(c) if c == expected => {
                let rest = &input[c.len_utf8()..];
                Ok((rest, c))
            }
            Some(c) => Err(format!("expected '{}', got '{}'", expected, c)),
            None => Err(format!("expected '{}', got end of input", expected)),
        }
    }
}

/// 述語を満たす文字1つにマッチするパーサーを生成する。
///
/// # 例
///
/// ```
/// use composition::satisfy;
///
/// let digit = satisfy(|c: char| c.is_ascii_digit());
/// assert_eq!(digit("123"), Ok(("23", '1')));
/// assert!(digit("abc").is_err());
/// ```
pub fn satisfy(predicate: impl Fn(char) -> bool) -> impl Fn(&str) -> ParseResult<char> {
    move |input: &str| {
        let mut chars = input.chars();
        match chars.next() {
            Some(c) if predicate(c) => {
                let rest = &input[c.len_utf8()..];
                Ok((rest, c))
            }
            Some(c) => Err(format!("unexpected char '{}'", c)),
            None => Err("unexpected end of input".to_string()),
        }
    }
}

/// 2つのパーサーを順に適用し、両方の結果をタプルで返すコンビネータ。
///
/// `pa` が成功した後、残りの入力に `pb` を適用する。
/// どちらかが失敗した時点でエラーを返す。
///
/// # 例
///
/// ```
/// use composition::{and_then_parser, char_parser};
///
/// let parse_ab = and_then_parser(char_parser('a'), char_parser('b'));
/// assert_eq!(parse_ab("abcd"), Ok(("cd", ('a', 'b'))));
/// assert!(parse_ab("axcd").is_err());
/// ```
pub fn and_then_parser<'a, A, B, PA, PB>(pa: PA, pb: PB) -> impl Fn(&'a str) -> ParseResult<'a, (A, B)>
where
    PA: Fn(&'a str) -> ParseResult<'a, A>,
    PB: Fn(&'a str) -> ParseResult<'a, B>,
{
    move |input: &'a str| {
        let (rest, a) = pa(input)?;
        let (rest2, b) = pb(rest)?;
        Ok((rest2, (a, b)))
    }
}

/// パーサーの結果に変換関数を適用するコンビネータ（Functor の `map` に相当）。
///
/// # 例
///
/// ```
/// use composition::{map_parser, char_parser};
///
/// let parse_a_as_str = map_parser(char_parser('a'), |c: char| c.to_string());
/// assert_eq!(parse_a_as_str("abc"), Ok(("bc", "a".to_string())));
/// ```
pub fn map_parser<'a, A, B, P, F>(parser: P, f: F) -> impl Fn(&'a str) -> ParseResult<'a, B>
where
    P: Fn(&'a str) -> ParseResult<'a, A>,
    F: Fn(A) -> B,
{
    move |input: &'a str| {
        let (rest, a) = parser(input)?;
        Ok((rest, f(a)))
    }
}

// ============================================================
// 4. NewType パターン
// ============================================================

/// ユーザーIDを表す型安全なラッパー。
///
/// 生の `u64` とは区別されるため、`PostId` などと混同するバグをコンパイル時に防げる。
///
/// # 例
///
/// ```
/// use composition::UserId;
///
/// let id = UserId(42);
/// assert_eq!(id.0, 42);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub u64);

/// 投稿IDを表す型安全なラッパー。
///
/// `UserId` と同じ内部表現を持つが、型は異なるため相互に代入できない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PostId(pub u64);

/// バリデーション済みのメールアドレスを表す型。
///
/// フィールドは非公開であり、`Email::new` を経由しないと生成できない。
/// これにより、`Email` 型の値は常に正しい形式であることが保証される。
///
/// # 例
///
/// ```
/// use composition::{Email, EmailError};
///
/// assert!(Email::new("user@example.com").is_ok());
/// assert_eq!(Email::new("invalid"), Err(EmailError::MissingAtSign));
/// assert_eq!(Email::new(""), Err(EmailError::Empty));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

/// `Email` の構築に失敗したときのエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmailError {
    /// 空文字列が渡された。
    Empty,
    /// `@` 記号が含まれていない。
    MissingAtSign,
    /// `@` の後にドメイン部分がない。
    MissingDomain,
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailError::Empty => write!(f, "email must not be empty"),
            EmailError::MissingAtSign => write!(f, "email must contain '@'"),
            EmailError::MissingDomain => write!(f, "email must have a domain after '@'"),
        }
    }
}

impl Email {
    /// メールアドレスの文字列を検証し、正しければ `Email` を生成する。
    ///
    /// 簡易的な検証として、`@` の存在と `@` 以降の文字列の有無を確認する。
    ///
    /// # Errors
    ///
    /// - `EmailError::Empty`: 空文字列が渡された場合
    /// - `EmailError::MissingAtSign`: `@` が含まれない場合
    /// - `EmailError::MissingDomain`: `@` の直後が文字列の末尾である場合
    pub fn new(raw: impl Into<String>) -> Result<Self, EmailError> {
        let s = raw.into();

        if s.is_empty() {
            return Err(EmailError::Empty);
        }

        let at_pos = s.find('@').ok_or(EmailError::MissingAtSign)?;

        if at_pos + 1 >= s.len() {
            return Err(EmailError::MissingDomain);
        }

        Ok(Email(s))
    }

    /// 内部のメールアドレス文字列への参照を返す。
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 正の整数のみを受け付ける型安全なラッパー。
///
/// # 例
///
/// ```
/// use composition::{PositiveInt, PositiveIntError};
///
/// assert!(PositiveInt::new(5).is_ok());
/// assert_eq!(PositiveInt::new(0), Err(PositiveIntError::NotPositive));
/// assert_eq!(PositiveInt::new(-1), Err(PositiveIntError::NotPositive));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositiveInt(i32);

/// `PositiveInt` の構築に失敗したときのエラー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PositiveIntError {
    /// 0以下の値が渡された。
    NotPositive,
}

impl std::fmt::Display for PositiveIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value must be a positive integer (> 0)")
    }
}

impl PositiveInt {
    /// 値を検証して `PositiveInt` を生成する。0以下の場合は `Err` を返す。
    ///
    /// # Errors
    ///
    /// - `PositiveIntError::NotPositive`: 値が 0 以下の場合
    pub fn new(value: i32) -> Result<Self, PositiveIntError> {
        if value > 0 {
            Ok(PositiveInt(value))
        } else {
            Err(PositiveIntError::NotPositive)
        }
    }

    /// 内部の値を返す。
    pub fn get(self) -> i32 {
        self.0
    }
}

// ============================================================
// テスト
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- compose / pipe ---

    #[test]
    fn test_compose_applies_f_then_g() {
        let add_one = |x: i32| x + 1;
        let double = |x: i32| x * 2;
        // compose(add_one, double)(3) = double(add_one(3)) = double(4) = 8
        let f = compose(add_one, double);
        assert_eq!(f(3), 8);
    }

    #[test]
    fn test_pipe_is_left_to_right() {
        let trim = |s: &str| s.trim().to_string();
        let to_upper = |s: String| s.to_uppercase();
        let normalize = pipe(trim, to_upper);
        assert_eq!(normalize("  hello  "), "HELLO");
    }

    #[test]
    fn test_pipe3_chains_three_functions() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;
        let h = |x: i32| x - 3;
        // (3+1)*2-3 = 5
        let pipeline = pipe3(f, g, h);
        assert_eq!(pipeline(3), 5);
    }

    #[test]
    fn test_pipe_all_macro_four_functions() {
        let f = |x: i32| x + 1;
        let g = |x: i32| x * 2;
        let h = |x: i32| x - 3;
        let i = |x: i32| x.to_string();
        // (3+1)*2-3 = 5 -> "5"
        let pipeline = pipe_all!(f, g, h, i);
        assert_eq!(pipeline(3), "5");
    }

    #[test]
    fn test_compose_identity_law() {
        // compose(id, f) == f
        let identity = |x: i32| x;
        let double = |x: i32| x * 2;
        let composed = compose(identity, double);
        for n in [0, 1, -5, 100] {
            assert_eq!(composed(n), double(n));
        }
    }

    // --- QueryBuilder ---

    #[test]
    fn test_query_builder_full() {
        let sql = QueryBuilder::new("users")
            .select("id")
            .select("name")
            .where_clause("age > 18")
            .where_clause("active = true")
            .order_by("name")
            .limit(10)
            .build();
        assert_eq!(
            sql,
            "SELECT id, name FROM users WHERE age > 18 AND active = true ORDER BY name LIMIT 10"
        );
    }

    #[test]
    fn test_query_builder_select_star_when_no_columns() {
        let sql = QueryBuilder::new("products").build();
        assert_eq!(sql, "SELECT * FROM products");
    }

    #[test]
    fn test_query_builder_without_optional_clauses() {
        let sql = QueryBuilder::new("orders").select("order_id").build();
        assert_eq!(sql, "SELECT order_id FROM orders");
    }

    // --- パーサーコンビネータ ---

    #[test]
    fn test_char_parser_success() {
        let parse_a = char_parser('a');
        assert_eq!(parse_a("abc"), Ok(("bc", 'a')));
    }

    #[test]
    fn test_char_parser_failure_wrong_char() {
        let parse_a = char_parser('a');
        assert!(parse_a("xyz").is_err());
    }

    #[test]
    fn test_char_parser_failure_empty_input() {
        let parse_a = char_parser('a');
        assert!(parse_a("").is_err());
    }

    #[test]
    fn test_satisfy_digit() {
        let digit = satisfy(|c: char| c.is_ascii_digit());
        assert_eq!(digit("123"), Ok(("23", '1')));
        assert!(digit("abc").is_err());
    }

    #[test]
    fn test_and_then_parser_success() {
        let parse_ab = and_then_parser(char_parser('a'), char_parser('b'));
        assert_eq!(parse_ab("abcd"), Ok(("cd", ('a', 'b'))));
    }

    #[test]
    fn test_and_then_parser_fails_on_second() {
        let parse_ab = and_then_parser(char_parser('a'), char_parser('b'));
        // 'a' は成功するが 'x' != 'b' なので失敗
        assert!(parse_ab("axcd").is_err());
    }

    #[test]
    fn test_map_parser_transforms_result() {
        let parse_a_upper = map_parser(char_parser('a'), |c: char| c.to_ascii_uppercase());
        assert_eq!(parse_a_upper("abc"), Ok(("bc", 'A')));
    }

    // --- NewType パターン ---

    #[test]
    fn test_user_id_and_post_id_are_distinct_types() {
        let uid = UserId(1);
        let pid = PostId(1);
        // 同じ値でも型が異なるため比較できない (コンパイル時に型チェックされる)
        // ここでは内部値が同じことだけ確認
        assert_eq!(uid.0, pid.0);
        // UserId と PostId は異なる型: uid == pid はコンパイルエラーになる
    }

    #[test]
    fn test_email_valid() {
        let email = Email::new("user@example.com");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_str(), "user@example.com");
    }

    #[test]
    fn test_email_empty_is_error() {
        assert_eq!(Email::new(""), Err(EmailError::Empty));
    }

    #[test]
    fn test_email_missing_at_sign_is_error() {
        assert_eq!(Email::new("invalidemail"), Err(EmailError::MissingAtSign));
    }

    #[test]
    fn test_email_missing_domain_is_error() {
        assert_eq!(Email::new("user@"), Err(EmailError::MissingDomain));
    }

    #[test]
    fn test_positive_int_valid() {
        let n = PositiveInt::new(42);
        assert!(n.is_ok());
        assert_eq!(n.unwrap().get(), 42);
    }

    #[test]
    fn test_positive_int_zero_is_error() {
        assert_eq!(PositiveInt::new(0), Err(PositiveIntError::NotPositive));
    }

    #[test]
    fn test_positive_int_negative_is_error() {
        assert_eq!(PositiveInt::new(-10), Err(PositiveIntError::NotPositive));
    }

    #[test]
    fn test_compose_and_newtype_together() {
        // NewType を使ったデータ変換パイプライン
        // UserId(u64) -> u64 -> String の変換を compose で繋ぐ
        let extract_id = |uid: UserId| uid.0;
        let format_id = |id: u64| format!("user:{}", id);
        let display_user = compose(extract_id, format_id);
        assert_eq!(display_user(UserId(99)), "user:99");
    }
}
