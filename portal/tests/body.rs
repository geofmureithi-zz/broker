#![deny(warnings)]

use bytes::Buf;
use futures::TryStreamExt;
use portal::Filter;

#[tokio::test]
async fn matches() {
    let _ = pretty_env_logger::try_init();

    let concat = portal::body::bytes();

    let req = portal::test::request().path("/nothing-matches-me");

    assert!(req.matches(&concat).await);

    let p = portal::path("body");
    let req = portal::test::request().path("/body");

    let and = p.and(concat);

    assert!(req.matches(&and).await);
}

#[tokio::test]
async fn server_error_if_taking_body_multiple_times() {
    let _ = pretty_env_logger::try_init();

    let concat = portal::body::bytes();
    let double = concat.and(concat).map(|_, _| portal::reply());

    let res = portal::test::request().reply(&double).await;

    assert_eq!(res.status(), 500);
    assert_eq!(res.body(), "Request body consumed multiple times");
}

#[tokio::test]
async fn content_length_limit() {
    let _ = pretty_env_logger::try_init();

    let limit = portal::body::content_length_limit(30).map(portal::reply);

    let res = portal::test::request().reply(&limit).await;
    assert_eq!(res.status(), 411, "missing content-length returns 411");

    let res = portal::test::request()
        .header("content-length", "999")
        .reply(&limit)
        .await;
    assert_eq!(res.status(), 413, "over limit returns 413");

    let res = portal::test::request()
        .header("content-length", "2")
        .reply(&limit)
        .await;
    assert_eq!(res.status(), 200, "under limit succeeds");
}

#[tokio::test]
async fn json() {
    let _ = pretty_env_logger::try_init();

    let json = portal::body::json::<Vec<i32>>();

    let req = portal::test::request().body("[1, 2, 3]");

    let vec = req.filter(&json).await.unwrap();
    assert_eq!(vec, &[1, 2, 3]);

    let req = portal::test::request()
        .header("content-type", "application/json")
        .body("[3, 2, 1]");

    let vec = req.filter(&json).await.unwrap();
    assert_eq!(vec, &[3, 2, 1], "matches content-type");
}

#[tokio::test]
async fn json_rejects_bad_content_type() {
    let _ = pretty_env_logger::try_init();

    let json = portal::body::json::<Vec<i32>>().map(|_| portal::reply());

    let req = portal::test::request()
        .header("content-type", "text/xml")
        .body("[3, 2, 1]");

    let res = req.reply(&json).await;
    assert_eq!(
        res.status(),
        415,
        "bad content-type should be 415 Unsupported Media Type"
    );
}

#[tokio::test]
async fn json_invalid() {
    let _ = pretty_env_logger::try_init();

    let json = portal::body::json::<Vec<i32>>().map(|vec| portal::reply::json(&vec));

    let res = portal::test::request().body("lol#wat").reply(&json).await;
    assert_eq!(res.status(), 400);
    let prefix = b"Request body deserialize error: ";
    assert_eq!(&res.body()[..prefix.len()], prefix);
}

#[tokio::test]
async fn form() {
    let _ = pretty_env_logger::try_init();

    let form = portal::body::form::<Vec<(String, String)>>();

    let req = portal::test::request().body("foo=bar&baz=quux");

    let vec = req.filter(&form).await.unwrap();
    let expected = vec![
        ("foo".to_owned(), "bar".to_owned()),
        ("baz".to_owned(), "quux".to_owned()),
    ];
    assert_eq!(vec, expected);
}

#[tokio::test]
async fn form_rejects_bad_content_type() {
    let _ = pretty_env_logger::try_init();

    let form = portal::body::form::<Vec<(String, String)>>().map(|_| portal::reply());

    let req = portal::test::request()
        .header("content-type", "application/x-www-form-urlencoded")
        .body("foo=bar");

    let res = req.reply(&form).await;
    assert_eq!(res.status(), 200);

    let req = portal::test::request()
        .header("content-type", "text/xml")
        .body("foo=bar");
    let res = req.reply(&form).await;
    assert_eq!(
        res.status(),
        415,
        "bad content-type should be 415 Unsupported Media Type"
    );
}

#[tokio::test]
async fn form_allows_charset() {
    let _ = pretty_env_logger::try_init();

    let form = portal::body::form::<Vec<(String, String)>>();

    let req = portal::test::request()
        .header(
            "content-type",
            "application/x-www-form-urlencoded; charset=utf-8",
        )
        .body("foo=bar");

    let vec = req.filter(&form).await.unwrap();
    let expected = vec![("foo".to_owned(), "bar".to_owned())];
    assert_eq!(vec, expected);
}

#[tokio::test]
async fn form_invalid() {
    let _ = pretty_env_logger::try_init();

    let form = portal::body::form::<Vec<i32>>().map(|vec| portal::reply::json(&vec));

    let res = portal::test::request().body("nope").reply(&form).await;
    assert_eq!(res.status(), 400);
    let prefix = b"Request body deserialize error: ";
    assert_eq!(&res.body()[..prefix.len()], prefix);
}

#[tokio::test]
async fn stream() {
    let _ = pretty_env_logger::try_init();

    let stream = portal::body::stream();

    let body = portal::test::request()
        .body("foo=bar")
        .filter(&stream)
        .await
        .expect("filter() stream");

    let bufs: Result<Vec<_>, portal::Error> = body.try_collect().await;
    let bufs = bufs.unwrap();

    assert_eq!(bufs.len(), 1);
    assert_eq!(bufs[0].bytes(), b"foo=bar");
}
