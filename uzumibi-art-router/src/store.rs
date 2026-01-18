use std::{cell::RefCell, collections::HashMap};

use crate::vendor_art_tree::{Art, ByteString};

/// Route object
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

/// Routing store using Adaptive Radix Tree
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
    /// Create a new RouteStore
    pub fn new() -> Self {
        RouteStore {
            art: RefCell::new(Art::new()),
        }
    }

    /// Convert path to routing key
    /// Example: "users/:id/posts" -> "users/\xff", "posts"
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
                // Parameters like :id are converted to 255u8 (\xff)
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

        // Rejoin with / and convert to byte string
        (ByteString::new(&path_joined), param_name, rest)
    }

    /// Register a route
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
                        // Overwrite if there's an existing handler
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

    /// Search for a route matching the path
    pub fn get(&self, path: &str) -> Option<T> {
        let (route, _) = find_route_recursive(self, path, &mut HashMap::new());
        route.and_then(|route| match route {
            Route::Handler(handler) => Some(handler),
            _ => None,
        })
    }

    /// Search for a route matching the path (with params)
    pub fn get_with_params(&self, path: &str) -> (Option<T>, HashMap<String, String>) {
        let (route, params) = find_route_recursive(self, path, &mut HashMap::new());
        (
            route.and_then(|route| match route {
                Route::Handler(handler) => Some(handler),
                _ => None,
            }),
            params,
        )
    }
}

fn find_route_recursive<T: Clone>(
    store: &RouteStore<T>,
    path: &str,
    params: &mut HashMap<String, String>,
) -> (Option<Route<T>>, HashMap<String, String>) {
    // First, search as-is
    let key = ByteString::new(path.as_bytes());
    if let Some(route) = store.art.borrow().get(&key) {
        return match route {
            Route::Handler(v) => (Some(Route::Handler(v.clone())), params.clone()),
            Route::SubRoutes { .. } => {
                panic!("Unexpected SubRoutes found at exact match")
            }
        };
    }

    // If no match, replace from right to left with \xff and search
    let segments: Vec<&str> = path.split('/').collect();

    // Replace slugs one by one from right to left with \xff
    for right_idx in (0..segments.len()).rev() {
        let matched_segment = segments[right_idx];
        // Replace the right_idx-th segment with \xff
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

        // Create path up to right_idx
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
                Route::Handler(v) => return (Some(Route::Handler(v.clone())), params.clone()),
                Route::SubRoutes {
                    param_name,
                    store: sub_store,
                } => {
                    params.insert(param_name.clone(), matched_segment.to_string());
                    // Recursively search the remaining left part
                    if right_idx + 1 < segments.len() {
                        let left_segments = &segments[right_idx + 1..];
                        let left_path = format!("/{}", left_segments.join("/"));
                        return find_route_recursive(sub_store, &left_path, params);
                    } else {
                        // If nothing remains, search with "/"
                        return find_route_recursive(sub_store, "/", params);
                    }
                }
            }
        }
    }

    (None, params.clone())
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
        // :id is converted to \xff
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
    fn test_get_with_params() {
        let store = RouteStore::new();
        store.insert("/users/:id", Route::new("handler1"));
        store.insert("/accounts/:id/items/:item_id", Route::new("handler2"));
        store.insert("/about", Route::new("handler3"));

        let (h1, params1) = store.get_with_params("/users/123");
        assert_eq!(h1.unwrap(), "handler1");
        assert_eq!(params1.get("id").unwrap(), "123");

        let (h1_2, params1_2) = store.get_with_params("/users/123456");
        assert_eq!(h1_2.unwrap(), "handler1");
        assert_eq!(params1_2.get("id").unwrap(), "123456");

        let (h2, params2) = store.get_with_params("/accounts/123/items/456");
        assert_eq!(h2.unwrap(), "handler2");
        assert_eq!(params2.get("id").unwrap(), "123");
        assert_eq!(params2.get("item_id").unwrap(), "456");

        let (h3, params3) = store.get_with_params("/about");
        assert_eq!(h3.unwrap(), "handler3");
        assert!(params3.is_empty());

        let h4 = store.get_with_params("/nonexistent");
        assert!(h4.0.is_none());
    }

    #[test]
    fn test_get_complex() {
        let store = RouteStore::new();
        store.insert("/users/:id", Route::new("handler1_2"));
        store.insert("/users/:id/items/:item_id", Route::new("handler2_2"));

        let (h1, params1) = store.get_with_params("/users/123");
        assert_eq!(h1.unwrap(), "handler1_2");
        assert_eq!(params1.len(), 1);
        assert_eq!(params1.get("id").unwrap(), "123");

        let (h2, params2) = store.get_with_params("/users/123/items/456");
        assert_eq!(h2.unwrap(), "handler2_2");
        assert_eq!(params2.len(), 2);
        assert_eq!(params2.get("id").unwrap(), "123");
        assert_eq!(params2.get("item_id").unwrap(), "456");

        let h3 = store.get("/users/123/hoge");
        assert!(h3.is_none());
    }
}
