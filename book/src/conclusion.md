# まとめ

## 学習した概念

このガイドでは、Rustを使って以下の関数型プログラミングの概念を学びました：

| 概念 | 重要なポイント |
|------|--------------|
| **純粋関数** | 同じ入力→同じ出力、副作用なし |
| **不変性** | データを変更せず新しいデータを生成する |
| **高階関数** | `map`、`filter`、`fold` で宣言的にデータを処理する |
| **クロージャ** | 環境をキャプチャする匿名関数 |
| **イテレータ** | 遅延評価によるデータ処理パイプライン |
| **パターンマッチング** | `match` で網羅的に型を処理する |
| **Option/Result** | 型安全なNull・エラーハンドリング |

## 関数型プログラミングのメリット

1. **コードの予測可能性**: 副作用がなく、入出力が明確
2. **テストのしやすさ**: 純粋関数は入出力だけをテストすればよい
3. **並列処理への適性**: 不変データはスレッド安全
4. **コードの再利用性**: 高階関数による抽象化
5. **バグの減少**: 状態変更が少ないほどバグは起きにくい

## Rust における関数型プログラミングのベストプラクティス

```rust
// 宣言的なスタイルを好む
let result: Vec<i32> = data
    .iter()
    .filter(|&&n| n > 0)
    .map(|&n| n * 2)
    .collect();

// 不変性を活かす
let original = vec![1, 2, 3];
let transformed = original.iter().map(|&n| n + 1).collect::<Vec<_>>();

// Option/Result をチェーンする
fn process(input: &str) -> Result<String, String> {
    input
        .trim()
        .parse::<i32>()
        .map_err(|e| e.to_string())
        .map(|n| format!("Processed: {}", n * 2))
}
```

## 次のステップ

さらに深く学ぶには以下のリソースが参考になります：

- [The Rust Programming Language (日本語版)](https://doc.rust-jp.rs/book-ja/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Rustlings](https://github.com/rust-lang/rustlings)

## コードを実行する

```bash
# すべてのテストを実行
cargo test

# 各トピックのサンプルを実行
cargo run -p pure_functions
cargo run -p immutability
cargo run -p higher_order_functions
cargo run -p closures
cargo run -p iterators
cargo run -p pattern_matching
cargo run -p option_and_result
```
