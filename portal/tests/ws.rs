#![deny(warnings)]

use futures::{FutureExt, StreamExt};
use portal::Filter;

#[tokio::test]
async fn upgrade() {
    let _ = pretty_env_logger::try_init();

    let route = portal::ws().map(|ws: portal::ws::Ws| ws.on_upgrade(|_| async {}));

    // From https://tools.ietf.org/html/rfc6455#section-1.2
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    let accept = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";

    let resp = portal::test::request()
        .header("connection", "upgrade")
        .header("upgrade", "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", key)
        .reply(&route)
        .await;

    assert_eq!(resp.status(), 101);
    assert_eq!(resp.headers()["connection"], "upgrade");
    assert_eq!(resp.headers()["upgrade"], "websocket");
    assert_eq!(resp.headers()["sec-websocket-accept"], accept);

    let resp = portal::test::request()
        .header("connection", "keep-alive, Upgrade")
        .header("upgrade", "Websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", key)
        .reply(&route)
        .await;

    assert_eq!(resp.status(), 101);
}

#[tokio::test]
async fn fail() {
    let _ = pretty_env_logger::try_init();

    let route = portal::any().map(portal::reply);

    portal::test::ws()
        .handshake(route)
        .await
        .expect_err("handshake non-websocket route should fail");
}

#[tokio::test]
async fn text() {
    let _ = pretty_env_logger::try_init();

    let mut client = portal::test::ws()
        .handshake(ws_echo())
        .await
        .expect("handshake");

    client.send_text("hello portal").await;

    let msg = client.recv().await.expect("recv");
    assert_eq!(msg.to_str(), Ok("hello portal"));
}

#[tokio::test]
async fn binary() {
    let _ = pretty_env_logger::try_init();

    let mut client = portal::test::ws()
        .handshake(ws_echo())
        .await
        .expect("handshake");

    client.send(portal::ws::Message::binary(&b"bonk"[..])).await;
    let msg = client.recv().await.expect("recv");
    assert!(msg.is_binary());
    assert_eq!(msg.as_bytes(), &b"bonk"[..]);
}

#[tokio::test]
async fn closed() {
    let _ = pretty_env_logger::try_init();

    let route =
        portal::ws().map(|ws: portal::ws::Ws| ws.on_upgrade(|websocket| websocket.close().map(|_| ())));

    let mut client = portal::test::ws().handshake(route).await.expect("handshake");

    client.recv_closed().await.expect("closed");
}

#[tokio::test]
async fn limit_message_size() {
    let _ = pretty_env_logger::try_init();

    let echo = portal::ws().map(|ws: portal::ws::Ws| {
        ws.max_message_size(1024).on_upgrade(|websocket| {
            // Just echo all messages back...
            let (tx, rx) = websocket.split();
            rx.forward(tx).map(|result| {
                assert!(result.is_err());
                assert_eq!(
                    format!("{}", result.unwrap_err()).as_str(),
                    "Space limit exceeded: Message too big: 0 + 1025 > 1024"
                );
            })
        })
    });
    let mut client = portal::test::ws().handshake(echo).await.expect("handshake");

    client.send(portal::ws::Message::binary(vec![0; 1025])).await;
    client.send_text("hello portal").await;
    assert!(client.recv().await.is_err());
}

fn ws_echo() -> impl Filter<Extract = impl portal::Reply, Error = portal::Rejection> + Copy {
    portal::ws().map(|ws: portal::ws::Ws| {
        ws.on_upgrade(|websocket| {
            // Just echo all messages back...
            let (tx, rx) = websocket.split();
            rx.forward(tx).map(|_| ())
        })
    })
}
