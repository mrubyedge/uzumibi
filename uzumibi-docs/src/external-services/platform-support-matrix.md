# Platform Support Matrix

| Service | Cloudflare | Fastly | Spin | Cloud Run |
|---------|-----------|--------|------|-----------|
| **KV** | ✅ Workers KV | ✅ KV Store | ✅ KV Store | ❌ TBA |
| **Cache** | ✅ Cache API | ✅ Edge Cache | ❌ TBA | ❌ TBA |
| **Secret** | ✅ Secrets | ✅ Secret Store | ✅ Variables | ✅ Secret Manager |
| **ObjectStore** | ✅ R2 | ❌ TBA | ❌ TBA | ✅ Cloud Storage |
| **Queue** | ✅ Queues | ❌ TBA | ❌ TBA | ✅ Pub/Sub |
| **SQL** | ✅ D1 | ❌ TBA | ✅ SQLite | ✅ Cloud SQL |
| **Fetch** | ✅ fetch API | ✅ Backends | ✅ Outbound HTTP | ✅ HTTP |

Legend:
- ✅ Planned/Available
- ❌ Not Available/TBA
- TBA: To Be Announced
