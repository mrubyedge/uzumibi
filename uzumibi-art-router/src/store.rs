use std::{cell::RefCell, collections::HashMap};

use crate::vendor_art_tree::{Art, ByteString};

/// Marker byte for parameter segments (e.g., :id)
const PARAM_MARKER: u8 = 0xff;
/// Marker byte for wildcard segments (*)
const WILDCARD_MARKER: u8 = 0xfe;

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
    Wildcard(T),
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
    /// Wildcard: "api/*" -> "api/\xfe" (captures rest of path as :*)
    fn path_to_key(path: &str) -> (ByteString, &str, String, bool) {
        let segments: Vec<&str> = path.split('/').collect();
        let mut path = Vec::new();
        let mut path_terminated = false;
        let mut param_name = "";
        let mut rest = Vec::new();
        let mut is_wildcard = false;
        for segment in segments.iter() {
            if path_terminated {
                rest.push(segment.to_string());
            } else if *segment == "*" {
                // Wildcard: captures rest of path
                let marker = vec![WILDCARD_MARKER];
                path_terminated = true;
                is_wildcard = true;
                param_name = "*";
                path.push(marker);
            } else if let Some(stripped) = segment.strip_prefix(':') {
                // Parameters like :id are converted to PARAM_MARKER
                let marker = vec![PARAM_MARKER];
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

        let rest = if path_terminated && !is_wildcard {
            let mut s = "/".to_string();
            s.push_str(&rest.join("/"));
            s
        } else {
            "".to_string()
        };

        // Rejoin with / and convert to byte string
        (ByteString::new(&path_joined), param_name, rest, is_wildcard)
    }

    /// Register a route
    pub fn insert(&self, path: &str, route: Route<T>) {
        let (key, param_name, subkey, is_wildcard) = Self::path_to_key(path);

        // Wildcard routes are stored directly
        if is_wildcard {
            if let Route::Handler(handler) = route {
                self.art.borrow_mut().insert(key, Route::Wildcard(handler));
            }
            return;
        }

        if !subkey.is_empty() {
            if let Some(subroute) = self.art.borrow().get(&key) {
                match subroute {
                    Route::SubRoutes {
                        param_name: _,
                        store,
                    } => {
                        store.insert(&subkey, route);
                    }
                    Route::Handler(_) | Route::Wildcard(_) => {
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
            Route::Handler(handler) | Route::Wildcard(handler) => Some(handler),
            _ => None,
        })
    }

    /// Search for a route matching the path (with params)
    pub fn get_with_params(&self, path: &str) -> (Option<T>, HashMap<String, String>) {
        let (route, params) = find_route_recursive(self, path, &mut HashMap::new());
        (
            route.and_then(|route| match route {
                Route::Handler(handler) | Route::Wildcard(handler) => Some(handler),
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
            Route::Wildcard(v) => (Some(Route::Wildcard(v.clone())), params.clone()),
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
                modified_path_bytes.push(PARAM_MARKER);
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
                    v.push(PARAM_MARKER);
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
                Route::Wildcard(v) => {
                    // Wildcard captures the rest of the path from this segment onwards
                    let rest_path = segments[right_idx..].join("/");
                    params.insert("*".to_string(), rest_path);
                    return (Some(Route::Wildcard(v.clone())), params.clone());
                }
                Route::SubRoutes {
                    param_name,
                    store: sub_store,
                } => {
                    let mut sub_params = params.clone();
                    sub_params.insert(param_name.clone(), matched_segment.to_string());
                    // Recursively search the remaining left part
                    let (result, result_params) = if right_idx + 1 < segments.len() {
                        let left_segments = &segments[right_idx + 1..];
                        let left_path = format!("/{}", left_segments.join("/"));
                        find_route_recursive(sub_store, &left_path, &mut sub_params)
                    } else {
                        // If nothing remains, search with "/"
                        find_route_recursive(sub_store, "/", &mut sub_params)
                    };
                    // Only return if we found a match, otherwise continue to try wildcard
                    if result.is_some() {
                        return (result, result_params);
                    }
                }
            }
        }

        // Also try wildcard marker for this position
        let wildcard_search_path: Vec<u8> = segments[..=right_idx]
            .iter()
            .enumerate()
            .flat_map(|(i, segment)| {
                let mut v = Vec::new();
                if i > 0 || !segment.is_empty() {
                    v.push(b'/');
                }
                if i == right_idx {
                    v.push(WILDCARD_MARKER);
                } else {
                    v.extend_from_slice(segment.as_bytes());
                }
                v
            })
            .collect();

        let wildcard_key = ByteString::new(&wildcard_search_path);
        if let Some(route) = art_borrow.get(&wildcard_key)
            && let Route::Wildcard(v) = route
        {
            // Wildcard captures the rest of the path from this segment onwards
            let rest_path = segments[right_idx..].join("/");
            params.insert("*".to_string(), rest_path);
            return (Some(Route::Wildcard(v.clone())), params.clone());
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

    #[test]
    fn test_wildcard() {
        let store = RouteStore::new();
        store.insert("/api/*", Route::new("api_handler"));
        store.insert("/static/*", Route::new("static_handler"));
        store.insert("/about", Route::new("about_handler"));

        let h1 = store.get("/api/users").unwrap();
        assert_eq!(h1, "api_handler");

        let h2 = store.get("/api/users/123/posts").unwrap();
        assert_eq!(h2, "api_handler");

        let h3 = store.get("/static/css/style.css").unwrap();
        assert_eq!(h3, "static_handler");

        let h4 = store.get("/about").unwrap();
        assert_eq!(h4, "about_handler");

        let h5 = store.get("/other");
        assert!(h5.is_none());
    }

    #[test]
    fn test_wildcard_with_params() {
        let store = RouteStore::new();
        store.insert("/api/*", Route::new("api_handler"));

        let (h1, params1) = store.get_with_params("/api/users/123/posts");
        assert_eq!(h1.unwrap(), "api_handler");
        assert_eq!(params1.get("*").unwrap(), "users/123/posts");

        let (h2, params2) = store.get_with_params("/api/single");
        assert_eq!(h2.unwrap(), "api_handler");
        assert_eq!(params2.get("*").unwrap(), "single");
    }

    #[test]
    fn test_path_to_key_wildcard() {
        let key = RouteStore::<()>::path_to_key("/api/*");
        assert_eq!(key.0.to_bytes(), b"/api/\xfe");
        assert_eq!(key.1, "*");
        assert_eq!(key.2, "");
        assert!(key.3); // is_wildcard
    }
}
