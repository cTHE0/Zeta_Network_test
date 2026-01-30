use libp2p::{
    futures::StreamExt,
    gossipsub::{self, IdentTopic, MessageAuthenticity},
    identify,
    identity::Keypair,
    kad::{self, store::MemoryStore},
    mdns, noise, ping,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{error, info};

mod web_server;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub author: String,
    pub author_name: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Post(Post),
    Heartbeat,
    PeerJoined { peer_id: String, name: String },
    PeerLeft { peer_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub name: Option<String>,
    pub is_browser: bool,
}

#[derive(NetworkBehaviour)]
struct ZetaBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    kad: kad::Behaviour<MemoryStore>,
}

#[derive(Clone)]
pub struct NetworkState {
    pub peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    pub posts: Arc<RwLock<Vec<Post>>>,
    pub local_peer_id: PeerId,
    pub local_name: String,
    // Channel pour diffuser aux clients WebSocket
    pub ws_broadcast: broadcast::Sender<String>,
}

impl NetworkState {
    fn new(local_peer_id: PeerId, local_name: String) -> Self {
        let (ws_broadcast, _) = broadcast::channel(100);
        Self {
            peers: Arc::new(RwLock::new(HashMap::new())),
            posts: Arc::new(RwLock::new(Vec::new())),
            local_peer_id,
            local_name,
            ws_broadcast,
        }
    }

    pub async fn add_peer(&self, peer_info: PeerInfo) {
        let peer_id = peer_info.peer_id.clone();
        self.peers.write().await.insert(peer_id.clone(), peer_info);
        // Notifier les clients WebSocket
        let msg = serde_json::json!({
            "type": "peer_joined",
            "peer_id": peer_id
        });
        let _ = self.ws_broadcast.send(msg.to_string());
    }

    pub async fn remove_peer(&self, peer_id: &str) {
        self.peers.write().await.remove(peer_id);
        let msg = serde_json::json!({
            "type": "peer_left", 
            "peer_id": peer_id
        });
        let _ = self.ws_broadcast.send(msg.to_string());
    }

    pub async fn add_post(&self, post: Post) {
        let mut posts = self.posts.write().await;
        posts.insert(0, post.clone());
        if posts.len() > 1000 {
            posts.truncate(1000);
        }
        // Notifier les clients WebSocket
        let msg = serde_json::json!({
            "type": "new_post",
            "post": post
        });
        let _ = self.ws_broadcast.send(msg.to_string());
    }

    pub async fn broadcast_to_ws(&self, message: &str) {
        let _ = self.ws_broadcast.send(message.to_string());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .init();

    info!("🚀 Démarrage de Zeta2 - Réseau social décentralisé");

    let args: Vec<String> = std::env::args().collect();
    let is_relay = args.contains(&"--relay".to_string()) || args.contains(&"--server".to_string());
    let relay_addr: Option<String> = args.iter()
        .position(|x| x == "--relay-addr")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let username: Option<String> = args.iter()
        .position(|x| x == "--name")
        .and_then(|i| args.get(i + 1))
        .cloned();
    let web_port: u16 = args.iter()
        .position(|x| x == "--web-port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);

    info!("⚙️  Mode: {}", if is_relay { "RELAY (Serveur)" } else { "CLIENT" });

    // Charger ou générer les clés
    let key_file = "identity.key";
    let local_key = if Path::new(key_file).exists() {
        info!("🔐 Chargement des clés existantes...");
        let key_bytes = fs::read(key_file)?;
        Keypair::from_protobuf_encoding(&key_bytes)?
    } else {
        info!("🔑 Génération de nouvelles clés...");
        let key = Keypair::generate_ed25519();
        let key_bytes = key.to_protobuf_encoding()?;
        fs::write(key_file, key_bytes)?;
        info!("💾 Clés sauvegardées dans {}", key_file);
        key
    };
    
    let local_peer_id = PeerId::from(local_key.public());
    info!("🔑 Peer ID: {}", local_peer_id);
    
    let local_name = username.unwrap_or_else(|| format!("Peer-{}", &local_peer_id.to_string()[..8]));
    info!("👤 Nom: {}", local_name);

    info!("📝 Initialisation du swarm...");

    // Créer le swarm avec TCP
    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            info!("📝 Configuration Gossipsub...");
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Permissive)
                .build()
                .expect("Configuration Gossipsub valide");

            let mut gossipsub = gossipsub::Behaviour::new(
                MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .expect("Impossible de créer Gossipsub");

            let topic = IdentTopic::new("zeta2-social");
            gossipsub.subscribe(&topic).unwrap();

            info!("📝 Configuration Identify...");
            let identify = identify::Behaviour::new(identify::Config::new(
                "/zeta2/1.0.0".to_string(),
                key.public(),
            ).with_push_listen_addr_updates(true));

            info!("📝 Configuration Kademlia...");
            let kad = kad::Behaviour::new(local_peer_id, MemoryStore::new(local_peer_id));

            info!("📝 Configuration mDNS...");
            let mdns = mdns::Behaviour::new(mdns::Config::default(), local_peer_id)
                .expect("Impossible de créer mDNS");

            info!("📝 Configuration Ping...");
            let ping = ping::Behaviour::new(ping::Config::new().with_interval(Duration::from_secs(15)));

            Ok(ZetaBehaviour {
                gossipsub,
                mdns,
                ping,
                identify,
                kad,
            })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    info!("✅ Swarm créé avec succès");
    let topic = IdentTopic::new("zeta2-social");
    info!("📡 Abonné au topic: {}", topic);

    // Configurer les listeners
    info!("📝 Configuration des listeners...");
    
    if is_relay {
        info!("🖥️  Mode RELAY - Écoute TCP sur 0.0.0.0:4001");
        swarm.listen_on("/ip4/0.0.0.0/tcp/4001".parse()?)?;
    } else {
        info!("💻 Mode CLIENT - Ports aléatoires");
        swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        if let Some(ref addr) = relay_addr {
            if let Ok(relay_multiaddr) = addr.parse::<Multiaddr>() {
                info!("🔗 Connexion au relay: {}", relay_multiaddr);
                swarm.dial(relay_multiaddr)?;
            }
        }
    }

    let network_state = NetworkState::new(local_peer_id, local_name.clone());

    let relay_multiaddr: Option<Multiaddr> = relay_addr.as_ref().and_then(|a| a.parse().ok());
    let relay_peer_id: Option<PeerId> = relay_multiaddr.as_ref().and_then(|addr| {
        addr.iter().find_map(|p| {
            if let libp2p::multiaddr::Protocol::P2p(peer_id) = p {
                Some(peer_id)
            } else {
                None
            }
        })
    });

    // Channel pour les posts (depuis web vers P2P)
    let (post_tx, mut post_rx) = mpsc::unbounded_channel::<Post>();
    // Channel pour les messages des clients WebSocket vers P2P
    let (ws_to_p2p_tx, mut ws_to_p2p_rx) = mpsc::unbounded_channel::<NetworkMessage>();

    // Démarrer le serveur web avec WebSocket
    let web_state = network_state.clone();
    let web_name = local_name.clone();
    tokio::spawn(async move {
        if let Err(e) = web_server::start_server(web_state, post_tx, ws_to_p2p_tx, web_name, is_relay, web_port).await {
            error!("❌ Erreur serveur web: {}", e);
        }
    });

    info!("🎉 Zeta2 démarré! Interface web sur http://localhost:{}", web_port);
    info!("⏳ En attente des événements réseau...");

    // Timer pour reconnexion automatique (commence après 30s)
    let mut reconnect_interval = tokio::time::interval(Duration::from_secs(30));
    reconnect_interval.tick().await; // Consommer le premier tick immédiat
    let mut connected_to_relay = false;

    // Boucle événements
    loop {
        tokio::select! {
            // Timer de reconnexion
            _ = reconnect_interval.tick() => {
                if !connected_to_relay {
                    if let Some(ref addr) = relay_multiaddr {
                        info!("🔄 Tentative de reconnexion au relay...");
                        if let Err(e) = swarm.dial(addr.clone()) {
                            error!("❌ Échec reconnexion: {}", e);
                        }
                    }
                }
            }
            // Message depuis WebSocket client vers P2P
            Some(ws_msg) = ws_to_p2p_rx.recv() => {
                if let Ok(json) = serde_json::to_vec(&ws_msg) {
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json) {
                        error!("❌ Erreur publication WS->P2P: {}", e);
                    } else {
                        info!("📤 Message WebSocket relayé au réseau P2P");
                        // Si c'est un post, l'ajouter localement aussi
                        if let NetworkMessage::Post(post) = ws_msg {
                            network_state.add_post(post).await;
                        }
                    }
                }
            }
            // Post depuis l'interface locale
            Some(post) = post_rx.recv() => {
                let msg = NetworkMessage::Post(post.clone());
                if let Ok(json) = serde_json::to_vec(&msg) {
                    if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json) {
                        error!("❌ Erreur publication: {}", e);
                    } else {
                        info!("📤 Post publié: {}", post.content);
                        network_state.add_post(post).await;
                    }
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("🎧 Écoute sur: {}/p2p/{}", address, local_peer_id);
                }
                SwarmEvent::Behaviour(ZetaBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message {
                        propagation_source: _peer_id,
                        message,
                        ..
                    },
                )) => {
                    if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                        match msg {
                            NetworkMessage::Post(post) => {
                                info!("📨 Nouveau post de {}: {}", post.author_name, post.content);
                                network_state.add_post(post).await;
                            }
                            NetworkMessage::PeerJoined { peer_id, name } => {
                                info!("👤 Peer {} ({}) a rejoint", name, peer_id);
                            }
                            NetworkMessage::PeerLeft { peer_id } => {
                                info!("👋 Peer {} a quitté", peer_id);
                            }
                            NetworkMessage::Heartbeat => {}
                        }
                    }
                }
                SwarmEvent::Behaviour(ZetaBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        info!("🔍 Peer découvert via mDNS: {}", peer_id);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        network_state.add_peer(PeerInfo {
                            peer_id: peer_id.to_string(),
                            address: multiaddr.to_string(),
                            name: None,
                            is_browser: false,
                        }).await;
                    }
                }
                SwarmEvent::Behaviour(ZetaBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _) in list {
                        info!("⏰ Peer expiré: {}", peer_id);
                        network_state.remove_peer(&peer_id.to_string()).await;
                    }
                }
                SwarmEvent::Behaviour(ZetaBehaviourEvent::Identify(identify::Event::Received {
                    peer_id,
                    info,
                    ..
                })) => {
                    info!("🆔 Peer identifié: {}", peer_id);
                    let addr = info.listen_addrs.first().map(|a| a.to_string()).unwrap_or_default();
                    network_state.add_peer(PeerInfo {
                        peer_id: peer_id.to_string(),
                        address: addr,
                        name: None,
                        is_browser: false,
                    }).await;
                }
                SwarmEvent::ConnectionEstablished { peer_id, num_established, .. } => {
                    info!("✅ Connexion: {} (total: {})", peer_id, num_established);
                    swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    if Some(peer_id) == relay_peer_id {
                        connected_to_relay = true;
                        info!("🔗 Connecté au relay!");
                    }
                }
                SwarmEvent::ConnectionClosed { peer_id, num_established, cause, .. } => {
                    info!("❌ Déconnexion: {} (restantes: {}) - Cause: {:?}", peer_id, num_established, cause);
                    if num_established == 0 {
                        network_state.remove_peer(&peer_id.to_string()).await;
                        if Some(peer_id) == relay_peer_id {
                            connected_to_relay = false;
                            info!("⚠️  Déconnecté du relay! Reconnexion dans 30s...");
                        }
                    }
                }
                SwarmEvent::IncomingConnection { local_addr, send_back_addr, .. } => {
                    info!("📥 Connexion entrante: {} -> {}", send_back_addr, local_addr);
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    error!("❌ Erreur connexion sortante vers {:?}: {}", peer_id, error);
                }
                SwarmEvent::Dialing { peer_id, .. } => {
                    info!("📞 Tentative de connexion à: {:?}", peer_id);
                }
                _ => {}
            }
        }
    }
}
