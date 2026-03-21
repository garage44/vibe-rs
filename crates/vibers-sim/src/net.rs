use crate::config::SimConfig;
use crate::state::SimWorld;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, RwLock};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use vibe_core::{
    decode_app_frame, encode_app_frame, NetMessage, ProtocolError, PROTOCOL_VERSION,
};

const MAX_FRAME: usize = 32 * 1024 * 1024;
/// ADR-012: simple per-connection rate limits (token-bucket style, fixed interval).
const MIN_INTENT_INTERVAL: Duration = Duration::from_millis(50);
const MIN_OBSERVER_INTERVAL: Duration = Duration::from_millis(100);

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
    let msg = decode_app_frame(&first)?;
    match msg {
        NetMessage::ClientHello {
            protocol_version,
            client_token,
        } => {
            if protocol_version != PROTOCOL_VERSION {
                let err = encode_app_frame(&NetMessage::ServerError {
                    request_id: 0,
                    code: 1,
                    message: format!("version {protocol_version} not supported"),
                })?;
                framed.send(Bytes::from(err)).await?;
                return Err(ProtocolError::UnsupportedVersion(protocol_version).into());
            }
            tracing::info!(token = %client_token, "client hello");
            let ack = encode_app_frame(&NetMessage::ServerHelloAck {
                session_id: uuid::Uuid::new_v4(),
                tick_hz: config.tick_hz,
                your_avatar_id: avatar_id,
                osm_tile_url_template: config.osm_tile_url_template.clone(),
            })?;
            framed.send(Bytes::from(ack)).await?;
        }
        other => {
            return Err(ProtocolError::ExpectedHello(format!("{other:?}").into()).into());
        }
    }

    let mut last_intent = Instant::now()
        .checked_sub(MIN_INTENT_INTERVAL)
        .unwrap_or_else(Instant::now);
    let mut last_observer = Instant::now()
        .checked_sub(MIN_OBSERVER_INTERVAL)
        .unwrap_or_else(Instant::now);

    loop {
        tokio::select! {
            biased;
            incoming = framed.next() => {
                match incoming {
                    None => break,
                    Some(Err(e)) => return Err(e.into()),
                    Some(Ok(bytes)) => {
                        let msg = decode_app_frame(&bytes)?;
                        match msg {
                            NetMessage::ClientIntent {
                                move_x,
                                move_z,
                                fly_up,
                                fly_down,
                                ..
                            } => {
                                if last_intent.elapsed() < MIN_INTENT_INTERVAL {
                                    continue;
                                }
                                last_intent = Instant::now();
                                let mut w = world.write().await;
                                w.apply_intent(avatar_id, move_x, move_z, fly_up, fly_down);
                            }
                            NetMessage::ObserverUpdate { position } => {
                                if last_observer.elapsed() < MIN_OBSERVER_INTERVAL {
                                    continue;
                                }
                                last_observer = Instant::now();
                                let mut w = world.write().await;
                                w.set_observer(position);
                            }
                            NetMessage::ClientHello { .. } => {
                                tracing::warn!("duplicate hello ignored");
                            }
                            NetMessage::PrimRemoved { .. }
                            | NetMessage::WorldSnapshot { .. }
                            | NetMessage::ServerHelloAck { .. }
                            | NetMessage::ServerError { .. } => {
                                tracing::debug!(?msg, "ignored message from client");
                            }
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

/// Periodically steps simulation and broadcasts postcard-encoded [`NetMessage::WorldSnapshot`] in app frames.
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
        match encode_app_frame(&snap) {
            Ok(bytes) => {
                let _ = tx.send(bytes);
            }
            Err(e) => tracing::error!("snapshot encode: {e}"),
        }
    }
}
