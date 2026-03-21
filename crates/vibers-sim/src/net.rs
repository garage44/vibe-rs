use crate::config::SimConfig;
use crate::state::SimWorld;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, RwLock};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use vibe_core::{decode_message, encode_message, NetMessage, ProtocolError, PROTOCOL_VERSION};

const MAX_FRAME: usize = 32 * 1024 * 1024;

pub async fn handle_connection(
    stream: TcpStream,
    world: Arc<RwLock<SimWorld>>,
    config: Arc<SimConfig>,
    mut snap_rx: broadcast::Receiver<Vec<u8>>,
    avatar_id: u64,
) -> anyhow::Result<()> {
    let mut framed = Framed::new(
        stream,
        LengthDelimitedCodec::builder()
            .max_frame_length(MAX_FRAME)
            .little_endian()
            .new_codec(),
    );

    let first = framed
        .next()
        .await
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("closed before hello"))?;
    let msg = decode_message(&first)?;
    match msg {
        NetMessage::ClientHello {
            protocol_version,
            client_token,
        } => {
            if protocol_version != PROTOCOL_VERSION {
                let err = encode_message(&NetMessage::ServerError {
                    request_id: 0,
                    code: 1,
                    message: format!("version {protocol_version} not supported"),
                })?;
                framed.send(Bytes::from(err)).await?;
                return Err(ProtocolError::UnsupportedVersion(protocol_version).into());
            }
            tracing::info!(token = %client_token, "client hello");
            let ack = encode_message(&NetMessage::ServerHelloAck {
                session_id: uuid::Uuid::new_v4(),
                tick_hz: config.tick_hz,
                your_avatar_id: avatar_id,
            })?;
            framed.send(Bytes::from(ack)).await?;
        }
        other => {
            return Err(ProtocolError::ExpectedHello(format!("{other:?}").into()).into());
        }
    }

    loop {
        tokio::select! {
            biased;
            incoming = framed.next() => {
                match incoming {
                    None => break,
                    Some(Err(e)) => return Err(e.into()),
                    Some(Ok(bytes)) => {
                        let msg = decode_message(&bytes)?;
                        match msg {
                            NetMessage::ClientIntent { move_x, move_z, fly_up, fly_down, .. } => {
                                let mut w = world.write().await;
                                w.apply_intent(avatar_id, move_x, move_z, fly_up, fly_down);
                            }
                            NetMessage::ObserverUpdate { position } => {
                                let mut w = world.write().await;
                                w.set_observer(position);
                            }
                            NetMessage::ClientHello { .. } => {
                                tracing::warn!("duplicate hello ignored");
                            }
                            _ => tracing::debug!(?msg, "ignored message from client"),
                        }
                    }
                }
            }
            snap = snap_rx.recv() => {
                match snap {
                    Ok(bytes) => {
                        framed.send(Bytes::from(bytes)).await?;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }
    Ok(())
}

/// Periodically steps simulation and broadcasts postcard-encoded [`NetMessage::WorldSnapshot`].
pub async fn tick_loop(
    world: Arc<RwLock<SimWorld>>,
    config: Arc<SimConfig>,
    tx: broadcast::Sender<Vec<u8>>,
) {
    let period = std::time::Duration::from_secs_f32((1.0 / config.tick_hz).max(0.001));
    let mut interval = tokio::time::interval(period);
    let mut tick: u64 = 0;
    loop {
        interval.tick().await;
        tick += 1;
        let mut w = world.write().await;
        w.step(period.as_secs_f32());
        let snap = w.snapshot(tick);
        drop(w);
        match encode_message(&snap) {
            Ok(bytes) => {
                let _ = tx.send(bytes);
            }
            Err(e) => tracing::error!("snapshot encode: {e}"),
        }
    }
}
