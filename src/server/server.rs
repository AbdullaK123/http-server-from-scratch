use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use crate::requests::HTTPRequest;
use crate::responses::HTTPResponse;
use crate::routing::{Router, Middleware};

pub struct HTTPServer {
    addr: String,
    routers: Vec<Router>,
    middleware: Vec<Middleware>
}

impl HTTPServer {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
            routers: Vec::new(),
            middleware: Vec::new()
        }
    }

    pub fn add_middleware(mut self, middleware: Middleware) -> Self {
        self.middleware.push(middleware);
        self
    }

    pub fn add_router(mut self, router: Router) -> Self {
        self.routers.push(router);
        self
    }

    async fn handle_connection(
        mut stream: TcpStream,
        routers: Arc<Vec<Router>>,
        middleware: Arc<Vec<Middleware>>
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut buffer = [0; 4096];
            let n_bytes = stream.read(&mut buffer).await?;

            if n_bytes == 0 {
                break; // Client disconnected
            }

            let request_str = String::from_utf8_lossy(&buffer[..n_bytes]);

            // Parse request
            let request = match HTTPRequest::new(&request_str) {
                Ok(req) => req,
                Err(e) => {
                    let res = HTTPResponse::new(400, &format!("Bad Request: {}", e));
                    stream.write_all(res.to_http_string().as_bytes()).await?;
                    continue;
                }
            };

            // handle global middleware chain
            let mut final_request: Result<HTTPRequest, HTTPResponse> = Ok(request.clone());
            for middleware in middleware.iter() {
                final_request = match final_request {
                    Ok(req) => (middleware)(req),
                    Err(res) => {
                        Err(res)
                    }
                }
            }
            let request_to_route = match final_request {
                Ok(req) => req,
                Err(res) => {
                    stream.write_all(res.to_http_string().as_bytes()).await?;
                    continue;
                }
            };

            // Try routers until one handles it
            let mut response = None;
            for router in routers.iter() {
                let res = router.handle_request(request_to_route.clone());
                if res.status.code() != 404 {
                    response = Some(res);
                    break;
                }
            }

            // Send response
            let final_response = response.unwrap_or_else(|| {
                HTTPResponse::not_found("No router matched this path")
            });

            stream.write_all(final_response.to_http_string().as_bytes()).await?;
        }

        Ok(())
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Started HTTP Server at {}", self.addr);

        let routers = Arc::new(self.routers);
        let middleware = Arc::new(self.middleware);

        loop {
            let (stream, addr) = listener.accept().await?;
            let routers = Arc::clone(&routers);
            let middleware = Arc::clone(&middleware);

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, routers, middleware).await {
                    eprintln!("Connection error from {}: {}", addr, e);
                }
            });
        }
    }
}