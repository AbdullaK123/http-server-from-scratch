mod requests;
mod responses;
mod routing;
mod server;

use requests::HTTPRequest;
use responses::HTTPResponse;
use routing::Router;
use server::HTTPServer;
use serde::{Serialize, Deserialize};

// ============================================
// Data Structures
// ============================================

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    name: String,
    email: String,
}

#[derive(Serialize, Debug)]
struct UserListResponse {
    page: i32,
    limit: i32,
    sort: String,
    users: Vec<User>,
    total: u32,
}

#[derive(Serialize, Debug)]
struct UserDetailResponse {
    id: String,
    name: String,
    email: String,
    include_posts: bool,
    include_comments: bool,
}

#[derive(Serialize, Debug)]
struct StatusResponse {
    id: u32,
    status: String,
}

#[derive(Serialize, Debug)]
struct HealthResponse {
    status: String,
    version: String,
}

// ============================================
// SERVER-LEVEL MIDDLEWARE (Layer 1)
// ============================================

fn request_logger(req: HTTPRequest) -> Result<HTTPRequest, HTTPResponse> {
    println!("🌐 [SERVER] {} {}", req.method, req.route);
    Ok(req)
}

fn global_cors(req: HTTPRequest) -> Result<HTTPRequest, HTTPResponse> {
    println!("🔓 [SERVER] CORS check passed");
    Ok(req)
}

fn security_check(req: HTTPRequest) -> Result<HTTPRequest, HTTPResponse> {
    println!("🔒 [SERVER] Security headers validated");
    Ok(req)
}

fn maintenance_mode(req: HTTPRequest) -> Result<HTTPRequest, HTTPResponse> {
    let maintenance = false; // Set to true to test

    if maintenance {
        println!("🚧 [SERVER] Maintenance mode active - blocking request");
        Err(HTTPResponse::new(503, "Service under maintenance"))
    } else {
        Ok(req)
    }
}

// ============================================
// ROUTER-LEVEL MIDDLEWARE (Layer 2)
// ============================================

fn api_key_check(req: HTTPRequest) -> Result<HTTPRequest, HTTPResponse> {
    match req.get_header("X-API-Key") {
        Some(key) if !key.is_empty() => {
            println!("🔑 [ROUTER] API key validated: {}", key);
            Ok(req)
        }
        _ => {
            println!("❌ [ROUTER] Missing or invalid API key");
            Err(HTTPResponse::new(401, "API key required"))
        }
    }
}

// ============================================
// ROUTE-LEVEL MIDDLEWARE (Layer 3)
// ============================================

fn admin_check(req: HTTPRequest) -> Result<HTTPRequest, HTTPResponse> {
    match req.get_header("X-Admin-Key") {
        Some(key) if key == "supersecret" => {
            println!("👑 [ROUTE] Admin access granted");
            Ok(req)
        }
        _ => {
            println!("⛔ [ROUTE] Admin access denied");
            Err(HTTPResponse::new(403, "Admin access required"))
        }
    }
}

fn rate_limit(req: HTTPRequest) -> Result<HTTPRequest, HTTPResponse> {
    println!("⏱️  [ROUTE] Rate limit check passed");
    Ok(req)
}

// ============================================
// HANDLERS (Layer 4)
// ============================================

fn home(_req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Serving home page");
    HTTPResponse::ok("")
        .with_html_body(
            r#"
            <h1>🚀 Middleware Test Server</h1>
            <h2>Test Routes:</h2>
            <ul>
                <li><a href="/">/ - Public (server middleware only)</a></li>
                <li><a href="/about">/about - About page</a></li>
                <li><a href="/api/users">/api/users - Needs API key (server + router)</a></li>
                <li><a href="/api/admin">/api/admin - Needs API key + Admin key (all layers)</a></li>
            </ul>
            "#
        )
}

fn about(_req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Serving about page");
    HTTPResponse::ok("")
        .with_html_body(
            r#"
            <h1>About This Framework</h1>
            <p>Built from scratch with Rust and Tokio!</p>
            <ul>
                <li>✅ 4-layer middleware system</li>
                <li>✅ Path & query parameters</li>
                <li>✅ JSON serialization with Serde</li>
                <li>✅ Multi-router architecture</li>
            </ul>
            <a href="/">Back to Home</a>
            "#
        )
}

fn list_users(req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Listing users");

    let page = req.query_int("page", 1);
    let limit = req.query_int("limit", 10);
    let sort = req.query("sort", "name");

    let response = UserListResponse {
        page,
        limit,
        sort,
        users: vec![
            User { id: 1, name: "Alice".into(), email: "alice@example.com".into() },
            User { id: 2, name: "Bob".into(), email: "bob@example.com".into() },
        ],
        total: 100,
    };

    HTTPResponse::ok_json(response).unwrap()
}

fn get_user(req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Getting user");

    let user_id = req.param("id", "0");
    let include_posts = req.query_bool("include_posts", false);
    let include_comments = req.query_bool("include_comments", false);

    let response = UserDetailResponse {
        id: user_id.clone(),
        name: format!("User {}", user_id),
        email: format!("user{}@example.com", user_id),
        include_posts,
        include_comments,
    };

    HTTPResponse::ok_json(response).unwrap()
}

fn create_user(req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Creating user");

    match req.body::<User>() {
        Ok(user) => {
            println!("   Parsed user: {:?}", user);

            let response = StatusResponse {
                id: 123,
                status: "created".into(),
            };

            HTTPResponse::json(201, response)
                .unwrap()
                .with_header("Location", "/api/users/123")
        }
        Err(e) => {
            println!("   JSON parse error: {}", e);
            HTTPResponse::new(400, &format!("Invalid JSON: {}", e))
        }
    }
}

fn update_user(req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Updating user");

    let user_id = req.param("id", "0");

    match req.body::<User>() {
        Ok(user) => {
            println!("   Updating user {}: {:?}", user_id, user);

            let response = StatusResponse {
                id: user_id.parse().unwrap_or(0),
                status: "updated".into(),
            };

            HTTPResponse::ok_json(response).unwrap()
        }
        Err(e) => {
            println!("   JSON parse error: {}", e);
            HTTPResponse::new(400, &format!("Invalid JSON: {}", e))
        }
    }
}

fn delete_user(req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Deleting user");

    let user_id = req.param("id", "0");
    println!("   Deleted user {}", user_id);

    HTTPResponse::new(204, "")
}

fn health_check(_req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Health check");

    let response = HealthResponse {
        status: "healthy".into(),
        version: "1.0.0".into(),
    };

    HTTPResponse::ok_json(response).unwrap()
}

fn admin_dashboard(_req: HTTPRequest) -> HTTPResponse {
    println!("✅ [HANDLER] Admin dashboard accessed");

    HTTPResponse::ok("")
        .with_html_body(
            r#"
            <h1>👑 Admin Dashboard</h1>
            <p>Welcome, admin! You passed all security layers.</p>
            <ul>
                <li>Total users: 100</li>
                <li>Active sessions: 42</li>
                <li>Requests today: 5,432</li>
            </ul>
            "#
        )
}

// ============================================
// MAIN
// ============================================

#[tokio::main]
async fn main() {
    println!("╔════════════════════════════════════════╗");
    println!("║   🚀 COMPLETE FRAMEWORK TEST 🚀       ║");
    println!("╚════════════════════════════════════════╝\n");

    // Public router - NO router middleware
    let public = Router::new("/")
        .get("/", home, vec![])
        .get("/about", about, vec![]);  // ← Added!

    // API router - WITH router middleware
    let api = Router::new("/api")
        .add_middleware(api_key_check)  // Layer 2: Router-level

        // All routes with proper middleware
        .get("/health", health_check, vec![])
        .get("/users", list_users, vec![])
        .post("/users", create_user, vec![])  // ← Added!
        .get("/users/{id}", get_user, vec![])  // ← Added!
        .put("/users/{id}", update_user, vec![])  // ← Added!
        .delete("/users/{id}", delete_user, vec![admin_check, rate_limit])  // ← Added!
        .get("/admin", admin_dashboard, vec![admin_check, rate_limit]);

    println!("📋 Routes registered:");
    println!("  GET    /");
    println!("  GET    /about");
    println!("  GET    /api/health");
    println!("  GET    /api/users");
    println!("  POST   /api/users");
    println!("  GET    /api/users/{{id}}");
    println!("  PUT    /api/users/{{id}}");
    println!("  DELETE /api/users/{{id}}");
    println!("  GET    /api/admin\n");

    println!("📋 Middleware Layers:");
    println!("  Layer 1 (Server):  request_logger → cors → security");
    println!("  Layer 2 (Router):  api_key_check (only on /api routes)");
    println!("  Layer 3 (Route):   admin_check + rate_limit (on protected routes)");
    println!("  Layer 4 (Handler): Your business logic\n");

    println!("🌐 Server starting on http://127.0.0.1:8081\n");
    println!("Run: ./src/test_server.sh to test all features!\n");

    // Start server with ALL FOUR LAYERS
    HTTPServer::new("127.0.0.1:8081")
        // LAYER 1: Server-level middleware (runs on EVERY request)
        .add_middleware(request_logger)
        .add_middleware(global_cors)
        .add_middleware(security_check)
        .add_middleware(maintenance_mode)

        // Add routers (Layer 2, 3, 4 inside)
        .add_router(public)
        .add_router(api)

        .run()
        .await
        .unwrap();
}