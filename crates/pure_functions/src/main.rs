/// 純粋関数（Pure Functions）の例
///
/// 純粋関数とは、以下の2つの性質を持つ関数です：
/// 1. 同じ引数を与えると必ず同じ結果を返す（参照透過性）
/// 2. 副作用（状態の変更、I/Oなど）を持たない

/// 純粋関数の例: 2つの数値を加算する
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// 純粋関数の例: 円の面積を計算する
pub fn circle_area(radius: f64) -> f64 {
    std::f64::consts::PI * radius * radius
}

/// 純粋関数の例: 文字列を大文字に変換する
pub fn to_uppercase(s: &str) -> String {
    s.to_uppercase()
}

/// 純粋関数の例: フィボナッチ数列のn番目の値を返す
pub fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    println!("=== 純粋関数（Pure Functions）===");

    // 加算
    println!("add(3, 4) = {}", add(3, 4));
    println!("add(3, 4) = {} (同じ入力、同じ出力)", add(3, 4));

    // 円の面積
    println!("circle_area(5.0) = {:.4}", circle_area(5.0));

    // 大文字変換
    println!("to_uppercase(\"hello\") = {}", to_uppercase("hello"));

    // フィボナッチ
    for i in 0..=10 {
        print!("{}", fibonacci(i));
        if i < 10 {
            print!(", ");
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_circle_area() {
        let area = circle_area(1.0);
        assert!((area - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_to_uppercase() {
        assert_eq!(to_uppercase("hello"), "HELLO");
        assert_eq!(to_uppercase("rust"), "RUST");
    }

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
    }

    /// 純粋関数の参照透過性を確認するテスト:
    /// 同じ引数で複数回呼び出しても、常に同じ結果が返ることを確認する
    #[test]
    fn test_referential_transparency() {
        let x = 7;
        let y = 3;
        let first_call = add(x, y);
        let second_call = add(x, y);
        assert_eq!(first_call, second_call);
    }
}
