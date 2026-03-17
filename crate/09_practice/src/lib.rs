//! # 第9章: 実践 — 小さな式インタープリタ
//!
//! 四則演算のみを扱う式インタープリタを関数型スタイルで実装します。
//!
//! 処理は以下の3層に分かれており、各層は独立した純粋関数です。
//!
//! ```text
//! 入力文字列
//!     │
//!     ▼
//! [トークナイザ]  tokenize()  →  Vec<Token>
//!     │
//!     ▼
//! [パーサ]        parse()     →  Expr (構文木)
//!     │
//!     ▼
//! [評価器]        eval()      →  f64
//! ```
//!
//! # 対応文法
//!
//! ```text
//! expr   = term  (('+' | '-') term)*
//! term   = number (('*' | '/') number)*
//! number = [0-9]+ ('.' [0-9]*)?
//! ```
//!
//! # 使用例
//!
//! ```
//! use practice::run;
//! assert_eq!(run("1 + 2 * 3").unwrap(), 7.0);
//! ```

// ---------------------------------------------------------------------------
// トークン
// ---------------------------------------------------------------------------

/// 字句解析器が生成するトークンの種類。
///
/// 入力文字列を意味を持つ最小単位（字句）に分解したものです。
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// 数値リテラル（浮動小数点数）
    Number(f64),
    /// 加算演算子 `+`
    Plus,
    /// 減算演算子 `-`
    Minus,
    /// 乗算演算子 `*`
    Star,
    /// 除算演算子 `/`
    Slash,
}

// ---------------------------------------------------------------------------
// 演算子・式（代数的データ型）
// ---------------------------------------------------------------------------

/// 二項演算子の種類。
///
/// `Expr::BinOp` の `op` フィールドとして使われ、
/// 評価器でのパターンマッチを網羅的に行うための型です。
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// 加算
    Add,
    /// 減算
    Sub,
    /// 乗算
    Mul,
    /// 除算
    Div,
}

/// 式の構文木（Abstract Syntax Tree）。
///
/// `Num` と `BinOp` の2種類の節で、任意に深い式を再帰的に表現します。
/// `Box` を使うことで再帰的な型定義をコンパイラに受け入れさせています。
///
/// # 例: `1 + 2 * 3` の構文木
///
/// ```text
/// BinOp(Add)
///   ├── Num(1.0)
///   └── BinOp(Mul)
///         ├── Num(2.0)
///         └── Num(3.0)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 数値リテラル
    Num(f64),
    /// 二項演算（左辺 `op` 右辺）
    BinOp {
        /// 演算子の種類
        op: Op,
        /// 左辺の式（再帰）
        lhs: Box<Expr>,
        /// 右辺の式（再帰）
        rhs: Box<Expr>,
    },
}

// ---------------------------------------------------------------------------
// 第1層: トークナイザ（純粋関数 + イテレータ）
// ---------------------------------------------------------------------------

/// 入力文字列をトークン列に変換する純粋関数。
///
/// - 空白（スペース・タブ・改行）はスキップします。
/// - 整数・浮動小数点数（例: `3`, `3.14`）を [`Token::Number`] に変換します。
/// - `+` `-` `*` `/` を対応するトークンに変換します。
/// - 認識できない文字が含まれる場合は `Err` を返します。
///
/// # 例
///
/// ```
/// use practice::{Token, tokenize};
/// let tokens = tokenize("1 + 2").unwrap();
/// assert_eq!(tokens, vec![Token::Number(1.0), Token::Plus, Token::Number(2.0)]);
/// ```
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // 空白はスキップ
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            // 演算子
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            // 数値: 先読みしながら全桁を収集する
            '0'..='9' | '.' => {
                // `from_fn` で Peekable の「次が数値なら消費する」処理をイテレータとして表現
                let num_str: String = std::iter::from_fn(|| {
                    chars
                        .peek()
                        .copied()
                        .filter(|ch| ch.is_ascii_digit() || *ch == '.')
                        .map(|ch| {
                            chars.next();
                            ch
                        })
                })
                .collect();

                let value = num_str
                    .parse::<f64>()
                    .map_err(|e| format!("数値 {num_str:?} のパースに失敗しました: {e}"))?;
                tokens.push(Token::Number(value));
            }
            other => {
                return Err(format!("認識できない文字: {other:?}"));
            }
        }
    }
    Ok(tokens)
}

// ---------------------------------------------------------------------------
// 第2層: パーサ（再帰下降 + Result）
// ---------------------------------------------------------------------------

/// トークン列を構文木（[`Expr`]）に変換する純粋関数。
///
/// 演算子優先順位（`*` `/` > `+` `-`）を再帰下降で処理します。
/// トークン列を完全に消費できない場合は `Err` を返します。
///
/// # 例
///
/// ```
/// use practice::{Token, Expr, Op, parse};
/// let tokens = vec![Token::Number(1.0), Token::Plus, Token::Number(2.0)];
/// let expr = parse(&tokens).unwrap();
/// assert_eq!(
///     expr,
///     Expr::BinOp {
///         op: Op::Add,
///         lhs: Box::new(Expr::Num(1.0)),
///         rhs: Box::new(Expr::Num(2.0)),
///     }
/// );
/// ```
pub fn parse(tokens: &[Token]) -> Result<Expr, String> {
    let (expr, rest) = parse_expr(tokens)?;
    if rest.is_empty() {
        Ok(expr)
    } else {
        Err(format!(
            "パース後に未処理のトークンが残っています: {rest:?}"
        ))
    }
}

/// 加減算レベル（優先順位低）の式をパースする。
///
/// `expr = term (('+' | '-') term)*` に対応。
/// 戻り値は `(解析済みの式, 未消費のトークンスライス)` のタプル。
fn parse_expr(tokens: &[Token]) -> Result<(Expr, &[Token]), String> {
    let (mut lhs, mut rest) = parse_term(tokens)?;

    // `+` または `-` が続く限り左結合で折り畳む
    while let Some(token) = rest.first() {
        let op = match token {
            Token::Plus => Op::Add,
            Token::Minus => Op::Sub,
            _ => break,
        };
        let (rhs, next) = parse_term(&rest[1..])?;
        lhs = Expr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
        rest = next;
    }
    Ok((lhs, rest))
}

/// 乗除算レベル（優先順位高）の式をパースする。
///
/// `term = number (('*' | '/') number)*` に対応。
fn parse_term(tokens: &[Token]) -> Result<(Expr, &[Token]), String> {
    let (mut lhs, mut rest) = parse_number(tokens)?;

    // `*` または `/` が続く限り左結合で折り畳む
    while let Some(token) = rest.first() {
        let op = match token {
            Token::Star => Op::Mul,
            Token::Slash => Op::Div,
            _ => break,
        };
        let (rhs, next) = parse_number(&rest[1..])?;
        lhs = Expr::BinOp {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
        rest = next;
    }
    Ok((lhs, rest))
}

/// 数値トークン1つをパースして [`Expr::Num`] を返す。
///
/// `split_first` でスライスの先頭要素と残りを同時に取り出す慣用句を使用。
fn parse_number(tokens: &[Token]) -> Result<(Expr, &[Token]), String> {
    match tokens.split_first() {
        Some((Token::Number(n), rest)) => Ok((Expr::Num(*n), rest)),
        Some((other, _)) => Err(format!(
            "数値トークンを期待しましたが {other:?} が見つかりました"
        )),
        None => Err("トークンが不足しています（数値を期待）".to_string()),
    }
}

// ---------------------------------------------------------------------------
// 第3層: 評価器（パターンマッチ + 再帰）
// ---------------------------------------------------------------------------

/// 構文木を再帰的に評価して `f64` を返す純粋関数。
///
/// - [`Expr::Num`] はそのまま値を返します。
/// - [`Expr::BinOp`] は左辺・右辺を再帰評価してから演算を適用します。
/// - ゼロ除算が発生した場合は `Err` を返します。
///
/// # 例
///
/// ```
/// use practice::{Expr, Op, eval};
/// let expr = Expr::BinOp {
///     op: Op::Mul,
///     lhs: Box::new(Expr::Num(3.0)),
///     rhs: Box::new(Expr::Num(4.0)),
/// };
/// assert_eq!(eval(&expr).unwrap(), 12.0);
/// ```
pub fn eval(expr: &Expr) -> Result<f64, String> {
    match expr {
        Expr::Num(n) => Ok(*n),

        Expr::BinOp { op, lhs, rhs } => {
            let l = eval(lhs)?; // 左辺を再帰評価（失敗したら即 Err を返す）
            let r = eval(rhs)?; // 右辺を再帰評価

            match op {
                Op::Add => Ok(l + r),
                Op::Sub => Ok(l - r),
                Op::Mul => Ok(l * r),
                Op::Div => {
                    if r == 0.0 {
                        Err("ゼロ除算エラー".to_string())
                    } else {
                        Ok(l / r)
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// パブリックエントリポイント
// ---------------------------------------------------------------------------

/// 入力文字列を字句解析・構文解析・評価して結果を返す関数。
///
/// 内部で [`tokenize`] → [`parse`] → [`eval`] を `?` で連結しています。
/// いずれかの層でエラーが発生した場合はそのエラーメッセージを `Err` として返します。
///
/// # 例
///
/// ```
/// use practice::run;
/// assert_eq!(run("1 + 2").unwrap(), 3.0);
/// assert_eq!(run("10 - 3").unwrap(), 7.0);
/// assert_eq!(run("2 * 6").unwrap(), 12.0);
/// assert_eq!(run("9 / 3").unwrap(), 3.0);
/// assert_eq!(run("1 + 2 * 3").unwrap(), 7.0);   // 優先順位: * > +
/// ```
///
/// 括弧はサポートしていないため `(10 - 4) / 2` のような入力はエラーになります。
pub fn run(input: &str) -> Result<f64, String> {
    let tokens = tokenize(input)?; // 第1層: 字句解析
    let expr = parse(&tokens)?;    // 第2層: 構文解析
    eval(&expr)                    // 第3層: 評価
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- トークナイザのテスト ---

    #[test]
    fn test_tokenize_simple_addition() {
        let tokens = tokenize("1 + 2").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Number(1.0), Token::Plus, Token::Number(2.0)]
        );
    }

    #[test]
    fn test_tokenize_all_operators() {
        let tokens = tokenize("1+2-3*4/5").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(1.0),
                Token::Plus,
                Token::Number(2.0),
                Token::Minus,
                Token::Number(3.0),
                Token::Star,
                Token::Number(4.0),
                Token::Slash,
                Token::Number(5.0),
            ]
        );
    }

    #[test]
    fn test_tokenize_float() {
        let tokens = tokenize("3.14 * 2.0").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Number(3.14), Token::Star, Token::Number(2.0)]
        );
    }

    #[test]
    fn test_tokenize_unknown_character() {
        assert!(tokenize("1 @ 2").is_err());
    }

    // --- 評価器のテスト（run 経由） ---

    #[test]
    fn test_run_addition() {
        assert_eq!(run("1 + 2").unwrap(), 3.0);
    }

    #[test]
    fn test_run_subtraction() {
        assert_eq!(run("10 - 3").unwrap(), 7.0);
    }

    #[test]
    fn test_run_multiplication() {
        assert_eq!(run("4 * 5").unwrap(), 20.0);
    }

    #[test]
    fn test_run_division() {
        assert_eq!(run("9 / 3").unwrap(), 3.0);
    }

    #[test]
    fn test_run_operator_precedence() {
        // 1 + 2 * 3 は 1 + (2 * 3) = 7 であり、(1 + 2) * 3 = 9 ではない
        assert_eq!(run("1 + 2 * 3").unwrap(), 7.0);
    }

    #[test]
    fn test_run_left_associativity() {
        // 10 - 3 - 2 は (10 - 3) - 2 = 5（左結合）
        assert_eq!(run("10 - 3 - 2").unwrap(), 5.0);
    }

    #[test]
    fn test_run_chained_multiplication() {
        // 2 * 3 * 4 = (2 * 3) * 4 = 24
        assert_eq!(run("2 * 3 * 4").unwrap(), 24.0);
    }

    #[test]
    fn test_run_mixed_operations() {
        // 100 - 2 * 10 + 5 = 100 - 20 + 5 = 85
        assert_eq!(run("100 - 2 * 10 + 5").unwrap(), 85.0);
    }

    #[test]
    fn test_run_float_arithmetic() {
        let result = run("1.5 + 2.5").unwrap();
        assert!((result - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_run_single_number() {
        assert_eq!(run("42").unwrap(), 42.0);
    }

    // --- エラーケースのテスト ---

    #[test]
    fn test_run_division_by_zero() {
        assert!(run("1 / 0").is_err());
        let err = run("1 / 0").unwrap_err();
        assert!(err.contains("ゼロ除算"), "エラーメッセージ: {err}");
    }

    #[test]
    fn test_run_invalid_character() {
        assert!(run("1 $ 2").is_err());
    }

    #[test]
    fn test_run_empty_input_is_err() {
        // トークンが空のためパースに失敗する
        assert!(run("").is_err());
    }

    #[test]
    fn test_run_trailing_operator_is_err() {
        // 末尾に演算子があるためパースに失敗する
        assert!(run("1 +").is_err());
    }

    // --- eval の直接テスト ---

    #[test]
    fn test_eval_nested_binop() {
        // (2 + 3) * (10 - 4) = 5 * 6 = 30
        let expr = Expr::BinOp {
            op: Op::Mul,
            lhs: Box::new(Expr::BinOp {
                op: Op::Add,
                lhs: Box::new(Expr::Num(2.0)),
                rhs: Box::new(Expr::Num(3.0)),
            }),
            rhs: Box::new(Expr::BinOp {
                op: Op::Sub,
                lhs: Box::new(Expr::Num(10.0)),
                rhs: Box::new(Expr::Num(4.0)),
            }),
        };
        assert_eq!(eval(&expr).unwrap(), 30.0);
    }
}
