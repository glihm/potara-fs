
use lazy_static::lazy_static;
use libp2p::{PeerId, gossipsub, mdns, tcp, yamux, noise, Transport};
use libp2p::identity::Keypair;
use libp2p::swarm::NetworkBehaviour;
use libp2p::core::upgrade;

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
}
