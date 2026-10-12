use std::time::Duration;

use rmcp::ServiceExt;
use tokio::{
    io::BufReader,
    net::{
        UnixListener, UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{mpsc, oneshot},
    time::timeout,
};

use super::AppServer;
use crate::{AccessGate, PeerInfo, handshake::read_token};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
const HANDSHAKE_QUEUE_CAPACITY: usize = 16;

struct Handshake {
    peer: PeerInfo,
    token: Option<String>,
    reader: BufReader<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

pub(super) async fn serve(
    listener: UnixListener,
    mut gate: Box<dyn AccessGate + Send>,
    server: AppServer,
    mut shutdown: oneshot::Receiver<()>,
) {
    let (handshake_sender, mut handshakes) = mpsc::channel(HANDSHAKE_QUEUE_CAPACITY);
    loop {
        tokio::select! {
            _ = &mut shutdown => break,
            accepted = listener.accept() => {
                if let Ok((stream, _)) = accepted {
                    tokio::spawn(read_handshake(stream, handshake_sender.clone()));
                }
            }
            Some(handshake) = handshakes.recv() => {
                if gate.authorize(&handshake.peer, handshake.token.as_deref()) {
                    tokio::spawn(run_session(handshake, server.clone()));
                }
            }
        }
    }
}

async fn read_handshake(stream: UnixStream, handshakes: mpsc::Sender<Handshake>) {
    let Ok(credentials) = stream.peer_cred() else {
        return;
    };
    let (reader, writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let Ok(Ok(token)) = timeout(HANDSHAKE_TIMEOUT, read_token(&mut reader)).await else {
        return;
    };
    let peer = PeerInfo {
        uid: credentials.uid(),
        pid: credentials.pid(),
    };
    let _ = handshakes
        .send(Handshake {
            peer,
            token,
            reader,
            writer,
        })
        .await;
}

async fn run_session(handshake: Handshake, server: AppServer) {
    let mut tool_updates = server.tool_updates();
    let Ok(session) = server.serve((handshake.reader, handshake.writer)).await else {
        return;
    };
    let peer = session.peer().clone();
    let announce_tool_changes = async move {
        while tool_updates.changed().await.is_ok() {
            let _ = peer.notify_tool_list_changed().await;
        }
    };
    tokio::select! {
        _ = session.waiting() => {}
        () = announce_tool_changes => {}
    }
}
