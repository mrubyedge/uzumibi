# uzumibi-on-cloudrun-spike

Cloud Run にデプロイ可能な hyper を使った HTTP サーバーのサンプルプロジェクト。

## ローカルでの実行

```bash
cargo run
```

デフォルトで http://0.0.0.0:8080 でリッスンします。

## Docker でのビルドと実行

```bash
# イメージのビルド
docker build -t uzumibi-cloudrun .

# コンテナの実行
docker run -p 8080:8080 uzumibi-cloudrun
```

## Cloud Run へのデプロイ

```bash
# Google Cloud プロジェクトIDを設定
export PROJECT_ID=your-project-id

# Cloud Build でイメージをビルドし、Container Registry にプッシュ
gcloud builds submit --tag gcr.io/$PROJECT_ID/uzumibi-cloudrun

# Cloud Run にデプロイ
gcloud run deploy uzumibi-cloudrun \
  --image gcr.io/$PROJECT_ID/uzumibi-cloudrun \
  --platform managed \
  --region asia-northeast1 \
  --allow-unauthenticated
```

## アーキテクチャ

- **hyper 1.5**: HTTP サーバーフレームワーク
- **tokio**: 非同期ランタイム
- **マルチステージビルド**: ビルドイメージと実行イメージを分離し、最終イメージサイズを最小化
