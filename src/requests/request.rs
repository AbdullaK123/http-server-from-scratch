use std::collections::HashMap;
use serde::{Deserialize};

#[derive(Debug, Clone)]
pub struct HTTPRequest {
    pub method: String,      // ← Not Option!
    pub route: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    body: String,
    pub route_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>
}

impl HTTPRequest {
    pub fn new(request: &str) -> Result<Self, String> {
        let parts: Vec<&str> = request.split("\r\n\r\n").collect();
        let body = parts.get(1).unwrap_or(&"").to_string();

        let (method, full_route, version) = Self::extract_method_route_and_version(request)?;
        let headers_map = Self::extract_headers(request);
        let (path, query_params) = Self::extract_query_params(full_route.as_str());

        Ok(Self {
            method,
            route: path,
            version,
            headers: headers_map,
            body,
            route_params: HashMap::new(), // for injecting route params
            query_params
        })
    }

    pub fn body<'a, T: Deserialize<'a>>(&'a self) -> Result<T, String> {
        serde_json::from_str(&self.body)
            .map_err(|e| format!("Failed to deserialize request body: {}", e))
    }

    // Get query param, returns owned String
    pub fn query(&self, key: &str, default: &str) -> String {
        self.query_params
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    // Get path param, returns owned String
    pub fn param(&self, key: &str, default: &str) -> String {
        self.route_params
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    // Get query param as i32
    pub fn query_int(&self, key: &str, default: i32) -> i32 {
        self.query_params
            .get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    // Get query param as bool
    pub fn query_bool(&self, key: &str, default: bool) -> bool {
        self.query_params
            .get(key)
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    // Check if query param exists
    pub fn has_query(&self, key: &str) -> bool {
        self.query_params.contains_key(key)
    }

    pub fn get_header(&self, header: &str) -> Option<String> {
        self.headers.get(header).cloned()
    }

    fn extract_query_params(full_route: &str)  -> (String, HashMap<String, String>) {
        if let Some((path, query_params_str)) = full_route.split_once("?") {
            let mut query_params = HashMap::new();

            for param in query_params_str.split("&") {
                if let Some((param_name, param_value)) = param.split_once("=") {
                    query_params.insert(param_name.to_string(), param_value.to_string());
                }
            }

            (path.to_string(), query_params)
        } else {
            (full_route.to_string(), HashMap::<String, String>::new())
        }
    }

    fn extract_method_route_and_version(request: &str) -> Result<(String, String, String), String>
    {
        let parts: Vec<&str> = request.split("\r\n\r\n").collect();
        let first_line_and_headers: Vec<&str> = parts[0].split("\r\n").collect();

        if first_line_and_headers.is_empty() {
            return Err("Empty request".to_string());
        }

        let first_line = first_line_and_headers[0];
        let first_line_parts: Vec<&str> = first_line.split_whitespace().collect();

        if first_line_parts.len() != 3 {
            return Err("Invalid request line format".to_string());
        }

        Ok((
            first_line_parts[0].to_string(),
            first_line_parts[1].to_string(),
            first_line_parts[2].to_string(),
        ))
    }

    fn extract_headers(request: &str) -> HashMap<String, String> {
        let parts: Vec<&str> = request.split("\r\n\r\n").collect();
        let first_line_and_headers: Vec<&str> = parts[0].split("\r\n").collect();
        let headers = &first_line_and_headers[1..];
        let mut headers_map = HashMap::new();

        for header in headers {
            if let Some((header, value)) = header.split_once(':') {
                headers_map.insert(
                    header.trim().to_string(),
                    value.trim().to_string(),
                );
            }
        }

        headers_map
    }
}