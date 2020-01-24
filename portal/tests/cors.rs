#![deny(warnings)]
use portal::{http::Method, Filter};

#[tokio::test]
async fn allow_methods() {
    let cors = portal::cors().allow_methods(&[Method::GET, Method::POST, Method::DELETE]);

    let route = portal::any().map(portal::reply).with(cors);

    let res = portal::test::request()
        .method("OPTIONS")
        .header("origin", "portal")
        .header("access-control-request-method", "DELETE")
        .reply(&route)
        .await;

    assert_eq!(res.status(), 200);

    let res = portal::test::request()
        .method("OPTIONS")
        .header("origin", "portal")
        .header("access-control-request-method", "PUT")
        .reply(&route)
        .await;

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn origin_not_allowed() {
    let cors = portal::cors()
        .allow_methods(&[Method::DELETE])
        .allow_origin("https://hyper.rs");

    let route = portal::any().map(portal::reply).with(cors);

    let res = portal::test::request()
        .method("OPTIONS")
        .header("origin", "https://portal.rs")
        .header("access-control-request-method", "DELETE")
        .reply(&route)
        .await;

    assert_eq!(res.status(), 403);

    let res = portal::test::request()
        .header("origin", "https://portal.rs")
        .header("access-control-request-method", "DELETE")
        .reply(&route)
        .await;

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn headers_not_exposed() {
    let cors = portal::cors()
        .allow_any_origin()
        .allow_methods(&[Method::GET]);

    let route = portal::any().map(portal::reply).with(cors);

    let res = portal::test::request()
        .method("OPTIONS")
        .header("origin", "https://portal.rs")
        .header("access-control-request-method", "GET")
        .reply(&route)
        .await;

    assert_eq!(
        res.headers().contains_key("access-control-expose-headers"),
        false
    );

    let res = portal::test::request()
        .method("GET")
        .header("origin", "https://portal.rs")
        .reply(&route)
        .await;

    assert_eq!(
        res.headers().contains_key("access-control-expose-headers"),
        false
    );
}

#[tokio::test]
async fn headers_not_allowed() {
    let cors = portal::cors()
        .allow_methods(&[Method::DELETE])
        .allow_headers(vec!["x-foo"]);

    let route = portal::any().map(portal::reply).with(cors);

    let res = portal::test::request()
        .method("OPTIONS")
        .header("origin", "https://portal.rs")
        .header("access-control-request-headers", "x-bar")
        .header("access-control-request-method", "DELETE")
        .reply(&route)
        .await;

    assert_eq!(res.status(), 403);
}

#[tokio::test]
async fn success() {
    let cors = portal::cors()
        .allow_credentials(true)
        .allow_headers(vec!["x-foo", "x-bar"])
        .allow_methods(&[Method::POST, Method::DELETE])
        .expose_header("x-header1")
        .expose_headers(vec!["x-header2"])
        .max_age(30);

    let route = portal::any().map(portal::reply).with(cors);

    // preflight
    let res = portal::test::request()
        .method("OPTIONS")
        .header("origin", "https://hyper.rs")
        .header("access-control-request-headers", "x-bar,x-foo")
        .header("access-control-request-method", "DELETE")
        .reply(&route)
        .await;
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.headers()["access-control-allow-origin"],
        "https://hyper.rs"
    );
    assert_eq!(res.headers()["access-control-allow-credentials"], "true");
    let allowed_headers = &res.headers()["access-control-allow-headers"];
    assert!(allowed_headers == "x-bar, x-foo" || allowed_headers == "x-foo, x-bar");
    let exposed_headers = &res.headers()["access-control-expose-headers"];
    assert!(exposed_headers == "x-header1, x-header2" || exposed_headers == "x-header2, x-header1");
    assert_eq!(res.headers()["access-control-max-age"], "30");
    let methods = &res.headers()["access-control-allow-methods"];
    assert!(
        // HashSet randomly orders these...
        methods == "DELETE, POST" || methods == "POST, DELETE",
        "access-control-allow-methods: {:?}",
        methods,
    );

    // cors request
    let res = portal::test::request()
        .method("DELETE")
        .header("origin", "https://hyper.rs")
        .header("x-foo", "hello")
        .header("x-bar", "world")
        .reply(&route)
        .await;
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.headers()["access-control-allow-origin"],
        "https://hyper.rs"
    );
    assert_eq!(res.headers()["access-control-allow-credentials"], "true");
    assert_eq!(res.headers().get("access-control-max-age"), None);
    assert_eq!(res.headers().get("access-control-allow-methods"), None);
    let exposed_headers = &res.headers()["access-control-expose-headers"];
    assert!(exposed_headers == "x-header1, x-header2" || exposed_headers == "x-header2, x-header1");
}

#[tokio::test]
async fn with_log() {
    let cors = portal::cors()
        .allow_any_origin()
        .allow_methods(&[Method::GET]);

    let route = portal::any()
        .map(portal::reply)
        .with(cors)
        .with(portal::log("cors test"));

    let res = portal::test::request()
        .method("OPTIONS")
        .header("origin", "portal")
        .header("access-control-request-method", "GET")
        .reply(&route)
        .await;

    assert_eq!(res.status(), 200);
}
