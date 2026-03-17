/// パターンマッチング（Pattern Matching）の例
///
/// パターンマッチングは関数型プログラミングの重要な機能です。
/// Rustの `match` 式は強力で網羅的なパターンマッチングを提供します。

/// 図形を表す列挙型
#[derive(Debug)]
pub enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

impl Shape {
    /// パターンマッチングで図形の面積を計算する
    pub fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }

    /// パターンマッチングで図形の名前を返す
    pub fn name(&self) -> &str {
        match self {
            Shape::Circle { .. } => "円",
            Shape::Rectangle { .. } => "長方形",
            Shape::Triangle { .. } => "三角形",
        }
    }
}

/// 式を表す再帰的な列挙型
#[derive(Debug)]
pub enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
}

impl Expr {
    /// パターンマッチングで式を評価する
    pub fn eval(&self) -> f64 {
        match self {
            Expr::Num(n) => *n,
            Expr::Add(a, b) => a.eval() + b.eval(),
            Expr::Mul(a, b) => a.eval() * b.eval(),
            Expr::Neg(e) => -e.eval(),
        }
    }
}

/// タプルのパターンマッチング
pub fn classify_point(x: i32, y: i32) -> &'static str {
    match (x, y) {
        (0, 0) => "原点",
        (x, 0) if x > 0 => "正のX軸上",
        (x, 0) if x < 0 => "負のX軸上",
        (0, y) if y > 0 => "正のY軸上",
        (0, y) if y < 0 => "負のY軸上",
        (x, y) if x > 0 && y > 0 => "第1象限",
        (x, y) if x < 0 && y > 0 => "第2象限",
        (x, y) if x < 0 && y < 0 => "第3象限",
        _ => "第4象限",
    }
}

/// if let を使ったパターンマッチング
pub fn get_even_double(n: i32) -> Option<i32> {
    if let Some(x) = Some(n).filter(|&x| x % 2 == 0) {
        Some(x * 2)
    } else {
        None
    }
}

fn main() {
    println!("=== パターンマッチング（Pattern Matching）===");

    // 図形の面積
    let shapes: Vec<Shape> = vec![
        Shape::Circle { radius: 3.0 },
        Shape::Rectangle { width: 4.0, height: 5.0 },
        Shape::Triangle { base: 6.0, height: 8.0 },
    ];

    for shape in &shapes {
        println!("{}: 面積 = {:.4}", shape.name(), shape.area());
    }

    // 式の評価: (2 + 3) * -(4)
    let expr = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Num(2.0)),
            Box::new(Expr::Num(3.0)),
        )),
        Box::new(Expr::Neg(Box::new(Expr::Num(4.0)))),
    );
    println!("(2 + 3) * -(4) = {}", expr.eval());

    // タプルのパターンマッチング
    let points = [(0, 0), (3, 0), (-2, 4), (1, -1)];
    for &(x, y) in &points {
        println!("({}, {}) -> {}", x, y, classify_point(x, y));
    }

    // if let
    for n in 1..=6 {
        match get_even_double(n) {
            Some(v) => println!("{} は偶数: 2倍 = {}", n, v),
            None => println!("{} は奇数", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_area() {
        let c = Shape::Circle { radius: 1.0 };
        assert!((c.area() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_rectangle_area() {
        let r = Shape::Rectangle { width: 4.0, height: 5.0 };
        assert_eq!(r.area(), 20.0);
    }

    #[test]
    fn test_triangle_area() {
        let t = Shape::Triangle { base: 6.0, height: 4.0 };
        assert_eq!(t.area(), 12.0);
    }

    #[test]
    fn test_expr_eval() {
        // 2 + 3 = 5
        let expr = Expr::Add(
            Box::new(Expr::Num(2.0)),
            Box::new(Expr::Num(3.0)),
        );
        assert!((expr.eval() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_expr_complex() {
        // (2 + 3) * -(4) = -20
        let expr = Expr::Mul(
            Box::new(Expr::Add(
                Box::new(Expr::Num(2.0)),
                Box::new(Expr::Num(3.0)),
            )),
            Box::new(Expr::Neg(Box::new(Expr::Num(4.0)))),
        );
        assert!((expr.eval() - (-20.0)).abs() < 1e-10);
    }

    #[test]
    fn test_classify_point() {
        assert_eq!(classify_point(0, 0), "原点");
        assert_eq!(classify_point(3, 0), "正のX軸上");
        assert_eq!(classify_point(1, 1), "第1象限");
        assert_eq!(classify_point(-1, -1), "第3象限");
    }

    #[test]
    fn test_get_even_double() {
        assert_eq!(get_even_double(4), Some(8));
        assert_eq!(get_even_double(3), None);
        assert_eq!(get_even_double(0), Some(0));
    }
}
