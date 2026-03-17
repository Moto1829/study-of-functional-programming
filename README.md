# study-of-functional-programming

Rustを用いた関数型プログラミングの勉強リポジトリです。

## 概要

このリポジトリは、**Rust** を使って **関数型プログラミング（Functional Programming）** の概念を学ぶためのものです。最終的に [mdBook](https://rust-lang.github.io/mdBook/) で公開することを目指しています。

## 学習トピック

1. **純粋関数（Pure Functions）** — 参照透過性と副作用のない関数
2. **不変性（Immutability）** — データを変更せず新しいデータを生成する
3. **高階関数（Higher-Order Functions）** — `map`、`filter`、`fold` による宣言的処理
4. **クロージャ（Closures）** — 環境をキャプチャする匿名関数
5. **イテレータ（Iterators）** — 遅延評価によるデータ処理パイプライン
6. **パターンマッチング（Pattern Matching）** — `match` による網羅的な型処理
7. **Option と Result** — 型安全なNull・エラーハンドリング

## プロジェクト構成

```
study-of-functional-programming/
├── Cargo.toml              # Rustワークスペース設定
├── book.toml               # mdBook設定
├── book/                   # mdBookドキュメント
│   └── src/
│       ├── SUMMARY.md
│       ├── introduction.md
│       ├── conclusion.md
│       └── chapters/       # 各トピックのドキュメント
└── crates/                 # Rustサンプルコード
    ├── pure_functions/
    ├── immutability/
    ├── higher_order_functions/
    ├── closures/
    ├── iterators/
    ├── pattern_matching/
    └── option_and_result/
```

## 使い方

### サンプルコードの実行

```bash
cargo run -p pure_functions
cargo run -p immutability
cargo run -p higher_order_functions
cargo run -p closures
cargo run -p iterators
cargo run -p pattern_matching
cargo run -p option_and_result
```

### テストの実行

```bash
cargo test
```

### ドキュメントのビルド

```bash
mdbook build book
```

### ドキュメントのローカルプレビュー

```bash
mdbook serve book
```

