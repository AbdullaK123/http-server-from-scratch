use crate::requests::HTTPRequest;
use crate::responses::HTTPResponse;

type Handler = fn(HTTPRequest) -> HTTPResponse;
pub type Middleware = fn(HTTPRequest) -> Result<HTTPRequest, HTTPResponse>;

#[derive(Clone)]
pub struct Route {
    method: String,
    path: String,
    handler: Handler,
    middleware: Vec<Middleware>
}

#[derive(Clone)]
pub struct Router {
    prefix: String,
    routes: Vec<Route>,
    middleware: Vec<Middleware>
}

impl Route {
    pub fn new(method: &str, path: &str, handler: Handler) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            handler,
            middleware: Vec::new()
        }
    }

    pub fn add_middleware(mut self, middleware: Middleware) -> Self {
        self.middleware.push(middleware);
        self
    }

    pub fn handle_request(&self, request: HTTPRequest) -> HTTPResponse {
        let mut final_request: Result<HTTPRequest, HTTPResponse> = Ok(request.clone());
        for middleware in &self.middleware {
            final_request = match final_request {
                Ok(req) => (middleware)(req),
                Err(res) => return res
            };
        }
        match final_request {
            Ok(req) => (self.handler)(req),
            Err(res) => res
        }
    }

    pub fn matches_route_pattern(&self, path: &str) -> bool {
        let pattern_parts: Vec<&str> = self.path.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        // Must have same number of segments
        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        // Compare each segment
        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            // Skip parameter placeholders like {id}
            if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                continue;
            }

            // Exact match required for non-parameter segments
            if pattern_part != path_part {
                return false;
            }
        }

        true
    }
}

impl Router {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            routes: Vec::new(),
            middleware: Vec::new()
        }
    }

    pub fn add_middleware(mut self, middleware: Middleware) -> Self {
        self.middleware.push(middleware);
        self
    }

    pub fn get(mut self, path: &str, handler: Handler, middleware: Vec<Middleware>) -> Self {
        let mut route = Route::new("GET", path, handler);
        for middleware in middleware {
            route = route.add_middleware(middleware);
        }
        self.routes.push(route);
        self
    }

    pub fn post(mut self, path: &str, handler: Handler, middleware: Vec<Middleware>) -> Self {
        let mut route = Route::new("POST", path, handler);
        for middleware in middleware {
            route = route.add_middleware(middleware);
        }
        self.routes.push(route);
        self
    }

    pub fn put(mut self, path: &str, handler: Handler, middleware: Vec<Middleware>) -> Self {
        let mut route = Route::new("PUT", path, handler);
        for middleware in middleware {
            route = route.add_middleware(middleware);
        }
        self.routes.push(route);
        self
    }

    pub fn patch(mut self, path: &str, handler: Handler, middleware: Vec<Middleware>) -> Self {
        let mut route = Route::new("PATCH", path, handler);
        for middleware in middleware {
            route = route.add_middleware(middleware);
        }
        self.routes.push(route);
        self
    }

    pub fn delete(mut self, path: &str, handler: Handler, middleware: Vec<Middleware>) -> Self {
        let mut route = Route::new("DELETE", path, handler);
        for middleware in middleware {
            route = route.add_middleware(middleware);
        }
        self.routes.push(route);
        self
    }

    fn inject_route_params_from_path(&self, request: &mut HTTPRequest, pattern: &str, actual_path: &str) {
        let path_parts: Vec<&str> = actual_path.split('/').collect();
        let pattern_parts: Vec<&str> = pattern.split('/').collect();

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if let Some(param_name) = pattern_part.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
                request.route_params.insert(param_name.to_string(), path_part.to_string());
            }
        }
    }

    pub fn handle_request(&self, mut request: HTTPRequest) -> HTTPResponse {
        let full_path = request.route.clone();

        // Strip prefix to get relative path
        let relative_path = if self.prefix == "/" {
            full_path.clone()
        } else {
            match full_path.strip_prefix(&self.prefix) {
                Some(p) => p.to_string(),
                None => return HTTPResponse::not_found("Route prefix not matched"),
            }
        };

        // Find matching route
        for route in &self.routes {
            if request.method == route.method && route.matches_route_pattern(&relative_path) {
                // CRITICAL FIX: Pass relative_path, not request.route!
                self.inject_route_params_from_path(&mut request, &route.path, &relative_path);

                let mut processed_request: Result<HTTPRequest, HTTPResponse> = Ok(request.clone());
                for middleware in &self.middleware {
                    processed_request = match processed_request {
                        Ok(req) => (middleware)(req),
                        Err(res) => return res
                    }
                }
                return match processed_request {
                    Ok(req) => route.handle_request(req),
                    Err(res) => res
                }
            }
        }

        HTTPResponse::not_found("No matching route found")
    }
}