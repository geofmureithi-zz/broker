#![deny(warnings)]
#[macro_use]
extern crate portal;

use futures::future;
use portal::Filter;

#[tokio::test]
async fn path() {
    let _ = pretty_env_logger::try_init();

    let foo = portal::path("foo");
    let bar = portal::path(String::from("bar"));
    let foo_bar = foo.and(bar.clone());

    // /foo
    let foo_req = || portal::test::request().path("/foo");

    assert!(foo_req().matches(&foo).await);
    assert!(!foo_req().matches(&bar).await);
    assert!(!foo_req().matches(&foo_bar).await);

    // /foo/bar
    let foo_bar_req = || portal::test::request().path("/foo/bar");

    assert!(foo_bar_req().matches(&foo).await);
    assert!(!foo_bar_req().matches(&bar).await);
    assert!(foo_bar_req().matches(&foo_bar).await);
}

#[tokio::test]
async fn param() {
    let _ = pretty_env_logger::try_init();

    let num = portal::path::param::<u32>();

    let req = portal::test::request().path("/321");
    assert_eq!(req.filter(&num).await.unwrap(), 321);

    let s = portal::path::param::<String>();

    let req = portal::test::request().path("/portal");
    assert_eq!(req.filter(&s).await.unwrap(), "portal");

    // u32 doesn't extract a non-int
    let req = portal::test::request().path("/portal");
    assert!(!req.matches(&num).await);

    let combo = num.map(|n| n + 5).and(s);

    let req = portal::test::request().path("/42/vroom");
    assert_eq!(req.filter(&combo).await.unwrap(), (47, "vroom".to_string()));

    // empty segments never match
    let req = portal::test::request();
    assert!(
        !req.matches(&s).await,
        "param should never match an empty segment"
    );
}

#[tokio::test]
async fn end() {
    let _ = pretty_env_logger::try_init();

    let foo = portal::path("foo");
    let end = portal::path::end();
    let foo_end = foo.and(end);

    assert!(
        portal::test::request().path("/").matches(&end).await,
        "end() matches /"
    );

    assert!(
        portal::test::request()
            .path("http://localhost:1234")
            .matches(&end)
            .await,
        "end() matches /"
    );

    assert!(
        portal::test::request()
            .path("http://localhost:1234?q=2")
            .matches(&end)
            .await,
        "end() matches empty path"
    );

    assert!(
        portal::test::request()
            .path("localhost:1234")
            .matches(&end)
            .await,
        "end() matches authority-form"
    );

    assert!(
        !portal::test::request().path("/foo").matches(&end).await,
        "end() doesn't match /foo"
    );

    assert!(
        portal::test::request().path("/foo").matches(&foo_end).await,
        "path().and(end()) matches /foo"
    );

    assert!(
        portal::test::request().path("/foo/").matches(&foo_end).await,
        "path().and(end()) matches /foo/"
    );
}

#[tokio::test]
async fn tail() {
    let tail = portal::path::tail();

    // matches full path
    let ex = portal::test::request()
        .path("/42/vroom")
        .filter(&tail)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "42/vroom");

    // matches index
    let ex = portal::test::request().path("/").filter(&tail).await.unwrap();
    assert_eq!(ex.as_str(), "");

    // doesn't include query
    let ex = portal::test::request()
        .path("/foo/bar?baz=quux")
        .filter(&tail)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "foo/bar");

    // doesn't include previously matched prefix
    let and = portal::path("foo").and(tail);
    let ex = portal::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "bar");

    // sets unmatched path index to end
    let m = tail.and(portal::path("foo"));
    assert!(!portal::test::request().path("/foo/bar").matches(&m).await);

    let m = tail.and(portal::path::end());
    assert!(portal::test::request().path("/foo/bar").matches(&m).await);
}

#[tokio::test]
async fn or() {
    let _ = pretty_env_logger::try_init();

    // /foo/bar OR /foo/baz
    let foo = portal::path("foo");
    let bar = portal::path("bar");
    let baz = portal::path("baz");
    let p = foo.and(bar.or(baz));

    // /foo/bar
    let req = portal::test::request().path("/foo/bar");

    assert!(req.matches(&p).await);

    // /foo/baz
    let req = portal::test::request().path("/foo/baz");

    assert!(req.matches(&p).await);

    // deeper nested ORs
    // /foo/bar/baz OR /foo/baz/bar OR /foo/bar/bar
    let p = foo
        .and(bar.and(baz).map(|| panic!("shouldn't match")))
        .or(foo.and(baz.and(bar)).map(|| panic!("shouldn't match")))
        .or(foo.and(bar.and(bar)));

    // /foo/baz
    let req = portal::test::request().path("/foo/baz/baz");
    assert!(!req.matches(&p).await);

    // /foo/bar/bar
    let req = portal::test::request().path("/foo/bar/bar");
    assert!(req.matches(&p).await);
}

#[tokio::test]
async fn or_else() {
    let _ = pretty_env_logger::try_init();

    let foo = portal::path("foo");
    let bar = portal::path("bar");

    let p = foo.and(bar.or_else(|_| future::ok::<_, std::convert::Infallible>(())));

    // /foo/bar
    let req = portal::test::request().path("/foo/nope");

    assert!(req.matches(&p).await);
}

#[tokio::test]
async fn path_macro() {
    let _ = pretty_env_logger::try_init();

    let req = portal::test::request().path("/foo/bar");
    let p = path!("foo" / "bar");
    assert!(req.matches(&p).await);

    let req = portal::test::request().path("/foo/bar");
    let p = path!(String / "bar");
    assert_eq!(req.filter(&p).await.unwrap(), "foo");

    let req = portal::test::request().path("/foo/bar");
    let p = path!("foo" / String);
    assert_eq!(req.filter(&p).await.unwrap(), "bar");

    // Requires path end

    let req = portal::test::request().path("/foo/bar/baz");
    let p = path!("foo" / "bar");
    assert!(!req.matches(&p).await);

    let req = portal::test::request().path("/foo/bar/baz");
    let p = path!("foo" / "bar").and(portal::path("baz"));
    assert!(!req.matches(&p).await);

    // Prefix syntax

    let req = portal::test::request().path("/foo/bar/baz");
    let p = path!("foo" / "bar" / ..);
    assert!(req.matches(&p).await);

    let req = portal::test::request().path("/foo/bar/baz");
    let p = path!("foo" / "bar" / ..).and(portal::path!("baz"));
    assert!(req.matches(&p).await);
}

#[tokio::test]
async fn full_path() {
    let full_path = portal::path::full();

    let foo = portal::path("foo");
    let bar = portal::path("bar");
    let param = portal::path::param::<u32>();

    // matches full request path
    let ex = portal::test::request()
        .path("/42/vroom")
        .filter(&full_path)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/42/vroom");

    // matches index
    let ex = portal::test::request()
        .path("/")
        .filter(&full_path)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/");

    // does not include query
    let ex = portal::test::request()
        .path("/foo/bar?baz=quux")
        .filter(&full_path)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/foo/bar");

    // includes previously matched prefix
    let and = foo.and(full_path);
    let ex = portal::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/foo/bar");

    // includes following matches
    let and = full_path.and(foo);
    let ex = portal::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/foo/bar");

    // includes previously matched param
    let and = foo.and(param).and(full_path);
    let (_, ex) = portal::test::request()
        .path("/foo/123")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "/foo/123");

    // does not modify matching
    let m = full_path.and(foo).and(bar);
    assert!(portal::test::request().path("/foo/bar").matches(&m).await);
}

#[tokio::test]
async fn peek() {
    let peek = portal::path::peek();

    let foo = portal::path("foo");
    let bar = portal::path("bar");
    let param = portal::path::param::<u32>();

    // matches full request path
    let ex = portal::test::request()
        .path("/42/vroom")
        .filter(&peek)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "42/vroom");

    // matches index
    let ex = portal::test::request().path("/").filter(&peek).await.unwrap();
    assert_eq!(ex.as_str(), "");

    // does not include query
    let ex = portal::test::request()
        .path("/foo/bar?baz=quux")
        .filter(&peek)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "foo/bar");

    // does not include previously matched prefix
    let and = foo.and(peek);
    let ex = portal::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "bar");

    // includes following matches
    let and = peek.and(foo);
    let ex = portal::test::request()
        .path("/foo/bar")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "foo/bar");

    // does not include previously matched param
    let and = foo.and(param).and(peek);
    let (_, ex) = portal::test::request()
        .path("/foo/123")
        .filter(&and)
        .await
        .unwrap();
    assert_eq!(ex.as_str(), "");

    // does not modify matching
    let and = peek.and(foo).and(bar);
    assert!(portal::test::request().path("/foo/bar").matches(&and).await);
}

#[tokio::test]
async fn peek_segments() {
    let peek = portal::path::peek();

    // matches full request path
    let ex = portal::test::request()
        .path("/42/vroom")
        .filter(&peek)
        .await
        .unwrap();

    assert_eq!(ex.segments().collect::<Vec<_>>(), &["42", "vroom"]);

    // matches index
    let ex = portal::test::request().path("/").filter(&peek).await.unwrap();

    let segs = ex.segments().collect::<Vec<_>>();
    assert_eq!(segs, Vec::<&str>::new());
}
