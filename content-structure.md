# Rustを使った関数型プログラミング — コンテンツ構成案

## 1. 関数型プログラミングの基礎概念

- 関数型プログラミングとは何か
- 命令型プログラミングとの違い
- 純粋関数（Pure Functions）
- 参照透過性（Referential Transparency）
- 副作用（Side Effects）とその管理

## 2. Rustにおける不変性と所有権

- デフォルト不変性（`let` vs `let mut`）
- 所有権と関数型プログラミングの親和性
- 借用とコピーセマンティクス
- `const` と `static`

## 3. クロージャと高階関数

- クロージャの基本構文
- `Fn` / `FnMut` / `FnOnce` トレイト
- 高階関数（関数を引数・戻り値にする）
- 関数ポインタとの違い

## 4. イテレータと遅延評価

- `Iterator` トレイトの仕組み
- `map` / `filter` / `fold` / `flat_map`
- イテレータチェーンによるデータ変換
- 遅延評価（Lazy Evaluation）と `collect`
- カスタムイテレータの実装

## 5. パターンマッチングと代数的データ型

- `enum` による代数的データ型（ADT）
- `match` 式とパターン分解
- `if let` / `while let`
- 直和型・直積型の考え方

## 6. Option型とResult型による関数型エラーハンドリング

- `Option<T>` — nullの代替
- `Result<T, E>` — 例外の代替
- `map` / `and_then` / `unwrap_or_else` による連鎖
- `?` 演算子の仕組み
- エラー型の設計

## 7. 関数合成とコンビネータパターン

- 関数合成の概念
- コンビネータパターン（Builder / Parser Combinator など）
- 型を使ったドメインモデリング

## 8. トレイトと型クラス的パターン

- トレイトと型クラスの対応
- Functor / Applicative / Monad 的なパターンをRustで表現する
- `Iterator` をMonadとして読む

## 9. 実践：関数型スタイルでの設計と実装

- 副作用の分離（純粋なコアとI/Oの境界）
- パイプラインスタイルのデータ処理
- 小さなインタープリタ実装（関数型設計の総合例）
