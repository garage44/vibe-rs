//! TCP client for `--connect` (ADR-008, ADR-009).

use crate::components::{Prim, PrimShape, Region};
use crate::resources::{AvatarState, ConnectAddr, GameState, NetworkMailbox, OnlineSession};
use bevy::prelude::*;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::sync::mpsc;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use vibe_core::{
    decode_message, encode_message, NetMessage, PrimDto, RegionDto, PROTOCOL_VERSION,
};

const MAX_FRAME: usize = 32 * 1024 * 1024;

pub fn spawn_network_thread(mut commands: Commands, addr: Res<ConnectAddr>) {
    let (out_tx, out_rx) = mpsc::channel::<NetMessage>();
    let (intent_tx, intent_rx) = tokio::sync::mpsc::unbounded_channel();
    let connect_to = addr.0.clone();
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("tokio runtime: {e}");
                return;
            }
        };
        if let Err(e) = rt.block_on(client_loop(connect_to, out_tx, intent_rx)) {
            eprintln!("network client ended: {e:#}");
        }
    });
    commands.insert_resource(NetworkMailbox {
        rx: std::sync::Mutex::new(out_rx),
    });
    commands.insert_resource(OnlineSession { intent_tx });
}

async fn client_loop(
    addr: String,
    out_tx: mpsc::Sender<NetMessage>,
    mut intent_rx: UnboundedReceiver<NetMessage>,
) -> anyhow::Result<()> {
    let stream = TcpStream::connect(&addr).await?;
    tracing::info!("connected to {addr}");
    let mut framed = Framed::new(
        stream,
        LengthDelimitedCodec::builder()
            .max_frame_length(MAX_FRAME)
            .little_endian()
            .new_codec(),
    );

    let hello = encode_message(&NetMessage::ClientHello {
        protocol_version: PROTOCOL_VERSION,
        client_token: format!("vibers-rs-{}", uuid::Uuid::new_v4()),
    })?;
    framed.send(Bytes::from(hello)).await?;

    let ack_bytes = framed
        .next()
        .await
        .transpose()?
        .ok_or_else(|| anyhow::anyhow!("closed before ServerHelloAck"))?;
    match decode_message(&ack_bytes)? {
        NetMessage::ServerHelloAck {
            tick_hz,
            your_avatar_id,
            ..
        } => {
            tracing::info!(tick_hz, your_avatar_id, "handshake ok");
        }
        NetMessage::ServerError { message, .. } => {
            anyhow::bail!("server error: {message}");
        }
        other => anyhow::bail!("unexpected first server message: {other:?}"),
    }

    loop {
        tokio::select! {
            biased;
            msg = intent_rx.recv() => {
                match msg {
                    Some(m) => {
                        let b = encode_message(&m)?;
                        framed.send(Bytes::from(b)).await?;
                    }
                    None => break,
                }
            }
            frame = framed.next() => {
                match frame {
                    None => break,
                    Some(Err(e)) => return Err(e.into()),
                    Some(Ok(bytes)) => {
                        let m = decode_message(&bytes)?;
                        if out_tx.send(m).is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Apply [`NetMessage::WorldSnapshot`] from the network thread (authoritative when online).
pub fn apply_network_snapshot(
    mut commands: Commands,
    mailbox: Option<Res<NetworkMailbox>>,
    mut game_state: ResMut<GameState>,
    mut avatar_state: ResMut<AvatarState>,
    region_entities: Query<Entity, With<Region>>,
    prim_entities: Query<Entity, With<Prim>>,
    mut avatar_tf: Query<&mut Transform, With<crate::components::Avatar>>,
) {
    let Some(mb) = mailbox else {
        return;
    };
    while let Ok(msg) = mb.lock_rx().try_recv() {
        let NetMessage::WorldSnapshot {
            regions,
            prims,
            avatars,
            tick,
        } = msg
        else {
            continue;
        };
        tracing::debug!(tick, "world snapshot");
        for e in region_entities.iter() {
            commands.entity(e).despawn();
        }
        for e in prim_entities.iter() {
            commands.entity(e).despawn();
        }
        game_state.regions_loaded = false;
        game_state.prims_loaded = false;

        for r in regions {
            commands.spawn(region_from_dto(r));
        }
        game_state.regions_loaded = true;

        for p in prims {
            commands.spawn(prim_bundle_from_dto(p));
        }
        game_state.prims_loaded = true;

        if let Some(a) = avatars.first() {
            if let Ok(mut tf) = avatar_tf.single_mut() {
                tf.translation = a.position;
            }
            avatar_state.position = a.position;
        }
    }
}

fn region_from_dto(r: RegionDto) -> Region {
    Region {
        id: r.id,
        name: r.name,
        latitude: r.latitude,
        longitude: r.longitude,
        tile_x: r.tile_x,
        tile_y: r.tile_y,
        tile_z: r.tile_z,
    }
}

fn prim_bundle_from_dto(p: PrimDto) -> (Prim, Transform) {
    (
        Prim {
            id: p.id,
            region_id: p.region_id,
            name: p.name,
            shape: PrimShape::from_str(&p.shape),
            color: Color::srgb(p.color[0], p.color[1], p.color[2]),
        },
        Transform::from_translation(p.position)
            .with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                p.rotation.x,
                p.rotation.y,
                p.rotation.z,
            ))
            .with_scale(p.scale),
    )
}

pub fn send_network_intent(
    online: Option<Res<OnlineSession>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    avatar_state: Res<AvatarState>,
    camera_state: Res<crate::resources::CameraState>,
) {
    let Some(sess) = online else {
        return;
    };
    if camera_state.mode == crate::resources::CameraMode::Free {
        return;
    }

    let move_forward =
        keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp);
    let move_backward =
        keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown);
    let move_left =
        keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft);
    let move_right =
        keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight);
    let fly_up = keyboard_input.pressed(KeyCode::Space);
    let fly_down = keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight);

    let mut v = Vec3::new(
        (move_right as i8 - move_left as i8) as f32,
        0.,
        (move_backward as i8 - move_forward as i8) as f32,
    );
    if v.length_squared() > 0.0001 {
        v = v.normalize();
        v = Mat3::from_rotation_y(avatar_state.rotation) * v;
    }

    let _ = sess.intent_tx.send(NetMessage::ClientIntent {
        request_id: 0,
        move_x: v.x,
        move_z: v.z,
        fly_up,
        fly_down,
    });
}

pub fn send_observer_update(
    online: Option<Res<OnlineSession>>,
    avatar_state: Res<AvatarState>,
    camera_state: Res<crate::resources::CameraState>,
) {
    let Some(sess) = online else {
        return;
    };
    if camera_state.mode == crate::resources::CameraMode::Free {
        return;
    }
    let _ = sess.intent_tx.send(NetMessage::ObserverUpdate {
        position: avatar_state.position,
    });
}
