/// 不変性（Immutability）の例
///
/// 関数型プログラミングでは、データを変更するのではなく
/// 新しいデータを生成することを好みます。
/// Rustでは変数はデフォルトで不変（immutable）です。

/// 不変な点を表す構造体
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    /// 元の点を変更せず、移動した新しい点を返す
    pub fn translate(&self, dx: f64, dy: f64) -> Point {
        Point {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// 元の点を変更せず、スケールした新しい点を返す
    pub fn scale(&self, factor: f64) -> Point {
        Point {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

/// 不変なリストを変換する例
/// 元のVecを変更せず、新しいVecを返す
pub fn double_all(numbers: &[i32]) -> Vec<i32> {
    numbers.iter().map(|&n| n * 2).collect()
}

/// 不変なリストから要素を追加した新しいリストを返す
pub fn append(numbers: &[i32], value: i32) -> Vec<i32> {
    let mut result = numbers.to_vec();
    result.push(value);
    result
}

fn main() {
    println!("=== 不変性（Immutability）===");

    // デフォルトで不変な変数
    let x = 5;
    println!("x = {}", x);
    // x = 6; // コンパイルエラー: cannot assign twice to immutable variable

    // 不変な構造体の変換
    let origin = Point::new(0.0, 0.0);
    let moved = origin.translate(3.0, 4.0);
    let scaled = moved.scale(2.0);

    println!("origin  = {:?}", origin); // 変更されていない
    println!("moved   = {:?}", moved);
    println!("scaled  = {:?}", scaled);

    // 不変なスライスの変換
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled = double_all(&numbers);
    let extended = append(&numbers, 6);

    println!("numbers  = {:?}", numbers); // 変更されていない
    println!("doubled  = {:?}", doubled);
    println!("extended = {:?}", extended);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_translate_does_not_mutate_original() {
        let origin = Point::new(1.0, 2.0);
        let _moved = origin.translate(5.0, 5.0);
        // 元の点が変化していないことを確認
        assert_eq!(origin, Point::new(1.0, 2.0));
    }

    #[test]
    fn test_point_translate() {
        let p = Point::new(0.0, 0.0);
        let moved = p.translate(3.0, 4.0);
        assert_eq!(moved, Point::new(3.0, 4.0));
    }

    #[test]
    fn test_point_scale() {
        let p = Point::new(2.0, 3.0);
        let scaled = p.scale(2.0);
        assert_eq!(scaled, Point::new(4.0, 6.0));
    }

    #[test]
    fn test_double_all_does_not_mutate_original() {
        let numbers = vec![1, 2, 3];
        let _doubled = double_all(&numbers);
        // 元のVecが変化していないことを確認
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    #[test]
    fn test_double_all() {
        assert_eq!(double_all(&[1, 2, 3]), vec![2, 4, 6]);
        assert_eq!(double_all(&[]), Vec::<i32>::new());
    }

    #[test]
    fn test_append() {
        let original = vec![1, 2, 3];
        let extended = append(&original, 4);
        assert_eq!(extended, vec![1, 2, 3, 4]);
        // 元のVecが変化していないことを確認
        assert_eq!(original, vec![1, 2, 3]);
    }
}
