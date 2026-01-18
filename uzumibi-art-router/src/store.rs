use std::cell::RefCell;

use crate::vendor_art_tree::{Art, ByteString};

/// ルートオブジェクト
pub enum Route<T>
where
    T: Clone,
{
    Handler(T),
    SubRoutes {
        param_name: String,
        store: Box<RouteStore<T>>,
    },
}

impl<T> Route<T>
where
    T: Clone,
{
    pub fn new(handler: T) -> Self {
        Route::Handler(handler)
    }

    pub fn sub_routes(param_name: impl Into<String>, store: RouteStore<T>) -> Self {
        Route::SubRoutes {
            param_name: param_name.into(),
            store: Box::new(store),
        }
    }
}

/// Adaptive Radix Treeを使ったルーティングストア
pub struct RouteStore<T>
where
    T: Clone,
{
    art: RefCell<Art<ByteString, Route<T>>>,
}

impl<T> RouteStore<T>
where
    T: Clone,
{
    /// 新しいRouteStoreを作成
    pub fn new() -> Self {
        RouteStore {
            art: RefCell::new(Art::new()),
        }
    }

    /// パスをルーティング用のキーに変換
    /// 例: "users/:id/posts" -> "users/\xff", "posts"
    fn path_to_key(path: &str) -> (ByteString, &str, String) {
        let segments: Vec<&str> = path.split('/').collect();
        let mut path = Vec::new();
        let mut path_terminated = false;
        let mut param_name = "";
        let mut rest = Vec::new();
        for segment in segments.iter() {
            if path_terminated {
                rest.push(segment.to_string());
            } else if let Some(stripped) = segment.strip_prefix(':') {
                // :id などのパラメータは 255u8 (\xff) に変換
                let marker = vec![0xffu8];
                path_terminated = true;
                param_name = stripped;
                path.push(marker);
            } else {
                path.push(segment.as_bytes().to_vec());
            }
        }

        let path_joined: Vec<u8> = path
            .iter()
            .flat_map(|s| {
                let mut v = Vec::new();
                v.extend_from_slice(s);
                v.push(b'/');
                v
            })
            .collect();

        let path_joined = if let Some(stripped) = path_joined.strip_suffix(b"/") {
            if stripped.is_empty() {
                b"/".to_vec()
            } else {
                stripped.to_vec()
            }
        } else {
            path_joined
        };

        let rest = if path_terminated {
            let mut s = "/".to_string();
            s.push_str(&rest.join("/"));
            s
        } else {
            "".to_string()
        };

        // /で再結合してバイト列に変換
        (ByteString::new(&path_joined), param_name, rest)
    }

    /// ルートを登録
    pub fn insert(&self, path: &str, route: Route<T>) {
        let (key, param_name, subkey) = Self::path_to_key(path);
        if !subkey.is_empty() {
            if let Some(subroute) = self.art.borrow().get(&key) {
                match subroute {
                    Route::SubRoutes {
                        param_name: _,
                        store,
                    } => {
                        store.insert(&subkey, route);
                    }
                    Route::Handler(_) => {
                        // 既存のハンドラがある場合は上書き
                        let store: RouteStore<T> = RouteStore::new();
                        store.insert(&subkey, route);
                        let new_subroute: Route<T> = Route::sub_routes(param_name, store);
                        self.art.borrow_mut().insert(key.clone(), new_subroute);
                    }
                }
            } else {
                let store: RouteStore<T> = RouteStore::new();
                store.insert(&subkey, route);
                let subroute: Route<T> = Route::sub_routes(param_name, store);
                self.art.borrow_mut().insert(key.clone(), subroute);
            }
        } else {
            self.art.borrow_mut().insert(key, route);
        }
    }

    /// パスに一致するルートを検索
    pub fn get(&self, path: &str) -> Option<T> {
        find_route_recursive(self, path).and_then(|route| match route {
            Route::Handler(handler) => Some(handler),
            _ => None,
        })
    }
}

fn find_route_recursive<T: Clone>(store: &RouteStore<T>, path: &str) -> Option<Route<T>> {
    // まずそのまま検索
    let key = ByteString::new(path.as_bytes());
    if let Some(route) = store.art.borrow().get(&key) {
        return match route {
            Route::Handler(v) => Some(Route::Handler(v.clone())),
            Route::SubRoutes { .. } => {
                panic!("Unexpected SubRoutes found at exact match")
            }
        };
    }

    // マッチしない場合、右から順に\xffに置き換えて検索
    let segments: Vec<&str> = path.split('/').collect();

    // 右から左へslugを一つずつ\xffに置き換える
    for right_idx in (0..segments.len()).rev() {
        // right_idx番目のセグメントを\xffに置き換える
        let mut modified_path_bytes = Vec::new();

        for (i, segment) in segments.iter().enumerate() {
            if i > 0 || !segment.is_empty() {
                modified_path_bytes.push(b'/');
            }

            if i == right_idx {
                modified_path_bytes.push(0xff);
            } else {
                modified_path_bytes.extend_from_slice(segment.as_bytes());
            }
        }

        // right_idxまでのパスを作成
        let search_path: Vec<u8> = segments[..=right_idx]
            .iter()
            .enumerate()
            .flat_map(|(i, segment)| {
                let mut v = Vec::new();
                if i > 0 || !segment.is_empty() {
                    v.push(b'/');
                }
                if i == right_idx {
                    v.push(0xff);
                } else {
                    v.extend_from_slice(segment.as_bytes());
                }
                v
            })
            .collect();

        let search_key = ByteString::new(&search_path);

        let art_borrow = store.art.borrow();
        if let Some(route) = art_borrow.get(&search_key) {
            match route {
                Route::Handler(v) => return Some(Route::Handler(v.clone())),
                Route::SubRoutes {
                    param_name: _,
                    store: sub_store,
                } => {
                    // 残りの左側の部分で再帰的に検索
                    if right_idx + 1 < segments.len() {
                        let left_segments = &segments[right_idx + 1..];
                        let left_path = format!("/{}", left_segments.join("/"));
                        return find_route_recursive(sub_store, &left_path);
                    } else {
                        // 残りがない場合は "/" で検索
                        return find_route_recursive(sub_store, "/");
                    }
                }
            }
        }
    }

    None
}

impl<T> Default for RouteStore<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::vendor_art_tree::Key;

    use super::*;

    #[test]
    fn test_path_to_key_simple() {
        let key = RouteStore::<()>::path_to_key("/users");
        assert_eq!(key.0.to_bytes(), b"/users");
        assert_eq!(key.1, "");
        assert_eq!(key.2, "");
    }

    #[test]
    fn test_path_to_key_with_param() {
        let key = RouteStore::<()>::path_to_key("/users/:id");
        // :id は \xff に変換される
        assert_eq!(key.0.to_bytes(), b"/users/\xff");
        assert_eq!(key.1, "id");
        assert_eq!(key.2, "/");
    }

    #[test]
    fn test_path_to_key_multiple_params() {
        let key = RouteStore::<()>::path_to_key("/users/:id/posts/:post_id");
        assert_eq!(key.0.to_bytes(), b"/users/\xff");
        assert_eq!(key.1, "id");
        assert_eq!(key.2, "/posts/:post_id");
    }

    #[test]
    fn test_insert() {
        let store = RouteStore::new();
        let route = Route::new(());

        store.insert("/users/:id", route);
        let key1 = ByteString::new(b"/users/\xff");
        let art_borrow = store.art.borrow();
        let v = art_borrow.get(&key1).unwrap();
        match v {
            Route::SubRoutes {
                param_name,
                store: _,
            } => {
                assert_eq!(param_name, "id");
            }
            _ => panic!("Expected SubRoutes"),
        }
    }

    #[test]
    fn test_insert_with_sub() {
        let store = RouteStore::new();
        let route = Route::new(());

        store.insert("/users/:id/followers/:follower_id", route);
        let key1 = ByteString::new(b"/users/\xff");
        let art_borrow = store.art.borrow();
        let v = art_borrow.get(&key1).unwrap();
        match v {
            Route::SubRoutes { param_name, store } => {
                assert_eq!(param_name, "id");
                let key2 = ByteString::new(b"/followers/\xff");
                let art_borrow2 = store.art.borrow();
                let v2 = art_borrow2.get(&key2).unwrap();
                match v2 {
                    Route::SubRoutes {
                        param_name: param_name2,
                        store: v3,
                    } => {
                        assert_eq!(param_name2, "follower_id");
                        let key3 = ByteString::new(b"/");
                        let art_borrow3 = v3.art.borrow();
                        let v4 = art_borrow3.get(&key3).unwrap();
                        match v4 {
                            Route::Handler(_) => { /* success */ }
                            _ => panic!("Expected Handler"),
                        }
                    }
                    _ => panic!("Expected SubRoutes v2"),
                }
            }
            _ => panic!("Expected SubRoutes"),
        }
    }

    #[test]
    fn test_get() {
        let store = RouteStore::new();
        store.insert("/users/:id", Route::new("handler1"));
        store.insert("/accounts/:id/items/:item_id", Route::new("handler2"));
        store.insert("/about", Route::new("handler3"));

        let h1 = store.get("/users/123").unwrap();
        assert_eq!(h1, "handler1");

        let h1_2 = store.get("/users/123456").unwrap();
        assert_eq!(h1_2, "handler1");

        let h2 = store.get("/accounts/123/items/456").unwrap();
        assert_eq!(h2, "handler2");

        let h3 = store.get("/about").unwrap();
        assert_eq!(h3, "handler3");

        let h4 = store.get("/nonexistent");
        assert!(h4.is_none());
    }

    #[test]
    fn test_get_complex() {
        let store = RouteStore::new();
        store.insert("/users/:id", Route::new("handler1_2"));
        store.insert("/users/:id/items/:item_id", Route::new("handler2_2"));

        let h1 = store.get("/users/123").unwrap();
        assert_eq!(h1, "handler1_2");

        let h2 = store.get("/users/123/items/456").unwrap();
        assert_eq!(h2, "handler2_2");

        let h3 = store.get("/users/123/hoge");
        assert!(h3.is_none());
    }
}
