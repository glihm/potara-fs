
use lazy_static::lazy_static;
use libp2p::{PeerId, gossipsub, mdns, tcp, yamux, noise, Transport};
use libp2p::identity::Keypair;
use libp2p::swarm::NetworkBehaviour;
use libp2p::core::upgrade;
use libp2p_quic as quic;

lazy_static! {
    static ref ID_KEYS: Keypair = Keypair::generate_ed25519();
    static ref PEER_ID: PeerId = PeerId::from(ID_KEYS.public());
}

#[derive(NetworkBehaviour)]
struct PotaraBehaviour {
    gossip_sub: gossipsub::Behaviour,
    mdns: mdns::async_io::Behaviour,
}

#[async_std::main]
async fn main() {
    env_logger::init();
    log::info!("Peer ID: {:?}", *PEER_ID);

    let _tcp_transport = tcp::async_io::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(upgrade::Version::V1Lazy)
        .authenticate(noise::Config::new(&ID_KEYS).expect("bad sign - keypair"))
        .multiplex(yamux::Config::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let quic_transport = quic::async_std::Transport::new(quic::Config::new(&id_keys));
    let transport = OrTransport::new(quic_transport, tcp_transport)
        .map(|either_output, _| match either_output {
            Either::Left((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
            Either::Right((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
        })
        .boxed();

    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    // Set a custom gossipsub configuration
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .build()
        .expect("invalid config");
}
