#![deny(warnings)]
use portal::Filter;

#[derive(Clone, Debug, PartialEq)]
struct Ext1(i32);

#[tokio::test]
async fn set_and_get() {
    let ext = portal::ext::get::<Ext1>();

    let extracted = portal::test::request()
        .extension(Ext1(55))
        .filter(&ext)
        .await
        .unwrap();

    assert_eq!(extracted, Ext1(55));
}

#[tokio::test]
async fn get_missing() {
    let ext = portal::ext::get().map(|e: Ext1| e.0.to_string());

    let res = portal::test::request().reply(&ext).await;

    assert_eq!(res.status(), 500);
    assert_eq!(res.body(), "Missing request extension");
}
