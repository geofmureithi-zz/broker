#![deny(warnings)]
use std::convert::Infallible;
use portal::Filter;

#[tokio::test]
async fn flattens_tuples() {
    let _ = pretty_env_logger::try_init();

    let str1 = portal::any().map(|| "portal");
    let true1 = portal::any().map(|| true);
    let unit1 = portal::any();

    // just 1 value
    let ext = portal::test::request().filter(&str1).await.unwrap();
    assert_eq!(ext, "portal");

    // just 1 unit
    let ext = portal::test::request().filter(&unit1).await.unwrap();
    assert_eq!(ext, ());

    // combine 2 values
    let and = str1.and(true1);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("portal", true));

    // combine 2 reversed
    let and = true1.and(str1);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, (true, "portal"));

    // combine 1 with unit
    let and = str1.and(unit1);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, "portal");

    let and = unit1.and(str1);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, "portal");

    // combine 3 values
    let and = str1.and(str1).and(true1);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("portal", "portal", true));

    // combine 2 with unit
    let and = str1.and(unit1).and(true1);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("portal", true));

    let and = unit1.and(str1).and(true1);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("portal", true));

    let and = str1.and(true1).and(unit1);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("portal", true));

    // nested tuples
    let str_true_unit = str1.and(true1).and(unit1);
    let unit_str_true = unit1.and(str1).and(true1);

    let and = str_true_unit.and(unit_str_true);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("portal", true, "portal", true));

    let and = unit_str_true.and(unit1).and(str1).and(str_true_unit);
    let ext = portal::test::request().filter(&and).await.unwrap();
    assert_eq!(ext, ("portal", true, "portal", "portal", true));
}

#[tokio::test]
async fn map() {
    let _ = pretty_env_logger::try_init();

    let ok = portal::any().map(portal::reply);

    let req = portal::test::request();
    let resp = req.reply(&ok).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn or() {
    let _ = pretty_env_logger::try_init();

    // Or can be combined with an infallible filter
    let a = portal::path::param::<u32>();
    let b = portal::any().map(|| 41i32);
    let f = a.or(b);

    let _: Result<_, Infallible> = portal::test::request().filter(&f).await;
}

#[tokio::test]
async fn or_else() {
    let _ = pretty_env_logger::try_init();

    let a = portal::path::param::<u32>();
    let f = a.or_else(|_| async { Ok::<_, portal::Rejection>((44u32,)) });

    assert_eq!(
        portal::test::request().path("/33").filter(&f).await.unwrap(),
        33,
    );
    assert_eq!(portal::test::request().filter(&f).await.unwrap(), 44,);

    // OrElse can be combined with an infallible filter
    let a = portal::path::param::<u32>();
    let f = a.or_else(|_| async { Ok::<_, Infallible>((44u32,)) });

    let _: Result<_, Infallible> = portal::test::request().filter(&f).await;
}

#[tokio::test]
async fn recover() {
    let _ = pretty_env_logger::try_init();

    let a = portal::path::param::<String>();
    let f = a.recover(|err| async move { Err::<String, _>(err) });

    // not rejected
    let resp = portal::test::request().path("/hi").reply(&f).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.body(), "hi");

    // rejected, recovered, re-rejected
    let resp = portal::test::request().reply(&f).await;
    assert_eq!(resp.status(), 404);

    // Recover can be infallible
    let f = a.recover(|_| async move { Ok::<_, Infallible>("shh") });

    let _: Result<_, Infallible> = portal::test::request().filter(&f).await;
}

#[tokio::test]
async fn unify() {
    let _ = pretty_env_logger::try_init();

    let a = portal::path::param::<u32>();
    let b = portal::path::param::<u32>();
    let f = a.or(b).unify();

    let ex = portal::test::request().path("/1").filter(&f).await.unwrap();

    assert_eq!(ex, 1);
}

#[should_panic]
#[tokio::test]
async fn nested() {
    let f = portal::any().and_then(|| {
        async {
            let p = portal::path::param::<u32>();
            portal::test::request().filter(&p).await
        }
    });

    let _ = portal::test::request().filter(&f).await;
}
