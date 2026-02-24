use std::collections::HashMap;

/// Parse x-www-form-urlencoded data into a HashMap
///
/// This function parses application/x-www-form-urlencoded format data
/// according to the WHATWG URL Standard.
///
/// # Format
/// - Key-value pairs are separated by '&'
/// - Keys and values are separated by '='
/// - Both keys and values are percent-encoded
/// - Space can be represented as '+' or '%20'
///
/// # Example
/// ```
/// use uzumibi_gem::helpers::parse_x_www_form_urlencoded;
/// 
/// let data = b"name=John+Doe&age=30&city=New%20York";
/// let params = parse_x_www_form_urlencoded(data);
/// assert_eq!(params.get("name"), Some(&"John Doe".to_string()));
/// assert_eq!(params.get("age"), Some(&"30".to_string()));
/// assert_eq!(params.get("city"), Some(&"New York".to_string()));
/// ```
pub fn parse_x_www_form_urlencoded(data: &[u8]) -> HashMap<String, String> {
    let mut result = HashMap::new();

    if data.is_empty() {
        return result;
    }

    // Convert bytes to string, replacing invalid UTF-8 sequences
    let data_str = String::from_utf8_lossy(data);

    // Split by '&' to get key-value pairs
    for pair in data_str.split('&') {
        if pair.is_empty() {
            continue;
        }

        // Split by '=' to separate key and value
        if let Some((key, value)) = pair.split_once('=') {
            let decoded_key = url_decode(key);
            let decoded_value = url_decode(value);
            result.insert(decoded_key, decoded_value);
        } else {
            // Handle keys without values (e.g., "key" or "key=")
            let decoded_key = url_decode(pair);
            result.insert(decoded_key, String::new());
        }
    }

    result
}

/// Decode a URL-encoded string
///
/// Decodes percent-encoded characters (%XX) and converts '+' to space.
fn url_decode(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '+' => result.push(' '),
            '%' => {
                // Try to read the next two hex digits
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        // Handle UTF-8 sequences
                        result.push(byte as char);
                    } else {
                        // Invalid hex sequence, keep as-is
                        result.push('%');
                        result.push_str(&hex);
                    }
                } else {
                    // Incomplete hex sequence
                    result.push('%');
                    result.push_str(&hex);
                }
            }
            _ => result.push(ch),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_data() {
        let result = parse_x_www_form_urlencoded(b"");
        assert!(result.is_empty());
    }

    #[test]
    fn test_simple_pairs() {
        let data = b"key1=value1&key2=value2";
        let result = parse_x_www_form_urlencoded(data);
        assert_eq!(result.get("key1"), Some(&"value1".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_plus_as_space() {
        let data = b"name=John+Doe&city=New+York";
        let result = parse_x_www_form_urlencoded(data);
        assert_eq!(result.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(result.get("city"), Some(&"New York".to_string()));
    }

    #[test]
    fn test_percent_encoding() {
        let data = b"name=John%20Doe&email=test%40example.com";
        let result = parse_x_www_form_urlencoded(data);
        assert_eq!(result.get("name"), Some(&"John Doe".to_string()));
        assert_eq!(result.get("email"), Some(&"test@example.com".to_string()));
    }

    #[test]
    fn test_special_characters() {
        let data = b"text=Hello%21%20World%3F";
        let result = parse_x_www_form_urlencoded(data);
        assert_eq!(result.get("text"), Some(&"Hello! World?".to_string()));
    }

    #[test]
    fn test_empty_value() {
        let data = b"key1=&key2=value2";
        let result = parse_x_www_form_urlencoded(data);
        assert_eq!(result.get("key1"), Some(&"".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_key_without_value() {
        let data = b"key1&key2=value2";
        let result = parse_x_www_form_urlencoded(data);
        assert_eq!(result.get("key1"), Some(&"".to_string()));
        assert_eq!(result.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_url_decode() {
        assert_eq!(url_decode("Hello+World"), "Hello World");
        assert_eq!(url_decode("Hello%20World"), "Hello World");
        assert_eq!(url_decode("test%40example.com"), "test@example.com");
        assert_eq!(url_decode("100%25"), "100%");
        assert_eq!(url_decode("a%2Bb%3Dc"), "a+b=c");
    }

    #[test]
    fn test_invalid_percent_encoding() {
        // Should handle invalid sequences gracefully
        assert_eq!(url_decode("test%"), "test%");
        assert_eq!(url_decode("test%1"), "test%1");
        assert_eq!(url_decode("test%GG"), "test%GG");
    }
}
