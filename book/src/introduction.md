# はじめに

このガイドは、**Rust** を使って **関数型プログラミング（Functional Programming）** の概念を学ぶためのリソースです。

## 関数型プログラミングとは？

関数型プログラミングは、計算を数学的な関数の評価として扱うプログラミングパラダイムです。以下の特徴を持ちます：

- **不変性（Immutability）**: データを変更するのではなく、新しいデータを生成します
- **純粋関数（Pure Functions）**: 同じ入力には必ず同じ出力を返し、副作用を持ちません
- **高階関数（Higher-Order Functions）**: 関数を引数や戻り値として扱います
- **宣言的スタイル**: 「何をするか」を記述し、「どのようにするか」の詳細を隠蔽します

## なぜ Rust で学ぶのか？

Rust は命令型言語でありながら、関数型プログラミングの多くの概念をサポートしています：

- 変数のデフォルト不変性
- 強力なイテレータと遅延評価
- クロージャのサポート
- パターンマッチング
- `Option<T>` と `Result<T, E>` による型安全なエラーハンドリング

## リポジトリ構成

```
study-of-functional-programming/
├── Cargo.toml              # Rustワークスペース設定
├── book.toml               # mdBook設定
├── book/                   # このドキュメント（mdBook）
│   └── src/
│       ├── SUMMARY.md
│       └── chapters/
└── crates/                 # Rustサンプルコード
    ├── pure_functions/
    ├── immutability/
    ├── higher_order_functions/
    ├── closures/
    ├── iterators/
    ├── pattern_matching/
    └── option_and_result/
```

## 動かし方

各クレートのサンプルコードを実行するには：

```bash
cargo run -p pure_functions
cargo run -p immutability
cargo run -p higher_order_functions
cargo run -p closures
cargo run -p iterators
cargo run -p pattern_matching
cargo run -p option_and_result
```

テストを実行するには：

```bash
cargo test
```

それでは、最初のトピック「純粋関数」から始めましょう！
