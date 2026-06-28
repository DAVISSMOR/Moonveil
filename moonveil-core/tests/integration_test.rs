use moonveil_core::{MuxError, Multiplexer, Session, TcpTransport};

#[tokio::test]
async fn test_client_server_roundtrip() {
    let addr = "127.0.0.1:19876";

    // Server: accept one connection, create session + multiplexer, receive 3 payloads.
    let server_task = tokio::spawn(async move {
        let listener = moonveil_core::TcpListener::bind(addr).await.unwrap();
        let server_transport = listener.accept().await.unwrap();

        let server_session = Session::new(Box::new(server_transport)).await;
        let server_session_id = server_session.id();

        let mux = Multiplexer::new().await;
        mux.add_session(server_session).await;

        let p1 = mux.recv_from(server_session_id).await.unwrap();
        let p2 = mux.recv_from(server_session_id).await.unwrap();
        let p3 = mux.recv_from(server_session_id).await.unwrap();

        assert_eq!(p1, b"ping1".to_vec());
        assert_eq!(p2, b"ping2".to_vec());
        assert_eq!(p3, b"ping3".to_vec());

        // graceful shutdown: remove session (ignore errors)
        let _ = mux.remove_session(server_session_id).await;
    });

    // Client: connect and send 3 packets.
    // Give the server a moment to bind.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let client_transport = TcpTransport::new(addr);
    let client_session = Session::new(Box::new(client_transport)).await;

    // We don't need a client-side multiplexer; just send on the transport.
    client_session
        .send(b"ping1".to_vec())
        .await
        .unwrap();
    client_session
        .send(b"ping2".to_vec())
        .await
        .unwrap();
    client_session
        .send(b"ping3".to_vec())
        .await
        .unwrap();

    let mut client_session_mut = client_session;
    let _ = client_session_mut.close().await;

    server_task.await.unwrap();
}

