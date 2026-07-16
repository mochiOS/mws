# mws

mwsは、repoで管理された複数のGitリポジトリをまとめて扱うためのワークスペースマネージャーです。

各リポジトリのコミットをワークスペース全体の状態として記録し、あとから同じ組み合わせへ復元できます。

### インストール

```bash
cargo install --path .
```

インストール後、`mws` がPATHから実行できることを確認します。

```bash
mws --help
```

### 初期化

repoワークスペースのルートで実行します。

```bash
mws init
```

mwsはrepo manifestからプロジェクトを読み込み、各リポジトリへGitフックを設定します。

### 状態を確認する

```bash
mws status
```

各リポジトリのHEADと未コミットの変更を確認します。

### 履歴を表示する

```bash
mws log
```

### 状態を復元する

```bash
mws restore <id>
```

最新の状態へ戻す場合、`latest`を指定します。

```bash
mws restore latest
```

未コミットの変更がある場合、復元は中止されます。

変更を破棄して復元する場合は`--force`を使用します。

```bash
mws restore latest --force
```

#### 過去のスナップショットからブランチを作る

```bash
mws restore <id> --work <branch_name>
```

デフォルトのrestoreはdetached headになりますが、これを利用することで全リポジトリにブランチを作成することができます。

### 作業ブランチ

作業ブランチの一覧を表示します。

```bash
mws work list
```

作業ブランチを削除します。

```bash
mws work clean <branch>
```

ブランチ名は`mws/`の有無にかかわらず指定できます。

### 保存データ

mwsのデータはワークスペースルートの`.workspace`に保存されます。

```text
.workspace/
├── tree.toml
└── snapshots/
```

mwsはリポジトリの内容を複製せず、各リポジトリのコミットIDを記録します。
