//! Zeta Network - Nœud P2P pour navigateur (WebAssembly)
//!
//! Ce module permet d'exécuter un nœud P2P directement dans le navigateur.
//! Il utilise WebSocket pour se connecter au relay P2P.

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket};

// ============================================
// Structures de données
// ============================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub author: String,
    pub author_name: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub name: Option<String>,
    pub is_browser: bool,
}

#[derive(Serialize, Deserialize)]
struct WsOutMessage {
    #[serde(rename = "type")]
    msg_type: String,
    content: Option<String>,
    author_name: Option<String>,
}

// ============================================
// État global du nœud
// ============================================

thread_local! {
    static NODE: RefCell<Option<ZetaNode>> = RefCell::new(None);
}

struct ZetaNode {
    peer_id: String,
    name: String,
    private_key: [u8; 32],
    ws: Option<WebSocket>,
    peers: HashMap<String, PeerInfo>,
    posts: Vec<Post>,
    on_message: Option<js_sys::Function>,
    on_peers: Option<js_sys::Function>,
    on_status: Option<js_sys::Function>,
}

// ============================================
// API JavaScript exposée
// ============================================

/// Initialise le nœud P2P dans le navigateur
#[wasm_bindgen]
pub fn init(relay_url: Option<String>) -> Result<JsValue, JsValue> {
    // Initialiser le panic hook pour de meilleurs messages d'erreur
    console_error_panic_hook::set_once();
    
    // Initialiser le logging
    tracing_wasm::set_as_global_default();
    
    log("🚀 Zeta Network WASM - Initialisation...");

    // Générer ou charger la clé
    let (peer_id, private_key) = load_or_generate_identity()?;
    let name = format!("Browser-{}", &peer_id[..8]);

    log(&format!("🆔 PeerId: {}", peer_id));
    log(&format!("👤 Nom: {}", name));

    // Créer le nœud
    let node = ZetaNode {
        peer_id: peer_id.clone(),
        name: name.clone(),
        private_key,
        ws: None,
        peers: HashMap::new(),
        posts: Vec::new(),
        on_message: None,
        on_peers: None,
        on_status: None,
    };

    NODE.with(|n| {
        *n.borrow_mut() = Some(node);
    });

    // Connexion au relay
    let relay = relay_url.unwrap_or_else(|| "ws://65.75.201.11:3030/ws".to_string());
    connect_to_relay(&relay)?;

    // Retourner les infos
    let info = serde_json::json!({
        "peer_id": peer_id,
        "name": name,
        "status": "initializing"
    });

    Ok(serde_wasm_bindgen::to_value(&info)?)
}

/// Définit le callback pour les nouveaux messages
#[wasm_bindgen]
pub fn on_message(callback: js_sys::Function) {
    NODE.with(|n| {
        if let Some(ref mut node) = *n.borrow_mut() {
            node.on_message = Some(callback);
        }
    });
}

/// Définit le callback pour les changements de peers
#[wasm_bindgen]
pub fn on_peers_change(callback: js_sys::Function) {
    NODE.with(|n| {
        if let Some(ref mut node) = *n.borrow_mut() {
            node.on_peers = Some(callback);
        }
    });
}

/// Définit le callback pour les changements de statut
#[wasm_bindgen]
pub fn on_status_change(callback: js_sys::Function) {
    NODE.with(|n| {
        if let Some(ref mut node) = *n.borrow_mut() {
            node.on_status = Some(callback);
        }
    });
}

/// Publie un post sur le réseau
#[wasm_bindgen]
pub fn publish_post(content: String, author_name: String) -> Result<JsValue, JsValue> {
    NODE.with(|n| {
        let mut node_ref = n.borrow_mut();
        let node = node_ref.as_mut().ok_or_else(|| JsValue::from_str("Node not initialized"))?;

        let post = Post {
            id: uuid::Uuid::new_v4().to_string(),
            author: node.peer_id.clone(),
            author_name,
            content,
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Envoyer via WebSocket
        if let Some(ref ws) = node.ws {
            if ws.ready_state() == WebSocket::OPEN {
                let msg = WsOutMessage {
                    msg_type: "post".to_string(),
                    content: Some(post.content.clone()),
                    author_name: Some(post.author_name.clone()),
                };
                let msg_str = serde_json::to_string(&msg)
                    .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
                ws.send_with_str(&msg_str)
                    .map_err(|e| JsValue::from_str(&format!("WebSocket send error: {:?}", e)))?;
            }
        }

        // Ajouter localement
        node.posts.insert(0, post.clone());

        log(&format!("📝 Post publié: {}", post.content));

        Ok(serde_wasm_bindgen::to_value(&post)?)
    })
}

/// Récupère la liste des peers
#[wasm_bindgen]
pub fn get_peers() -> Result<JsValue, JsValue> {
    NODE.with(|n| {
        let node_ref = n.borrow();
        let node = node_ref.as_ref().ok_or_else(|| JsValue::from_str("Node not initialized"))?;
        let peers: Vec<&PeerInfo> = node.peers.values().collect();
        Ok(serde_wasm_bindgen::to_value(&peers)?)
    })
}

/// Récupère la liste des posts
#[wasm_bindgen]
pub fn get_posts() -> Result<JsValue, JsValue> {
    NODE.with(|n| {
        let node_ref = n.borrow();
        let node = node_ref.as_ref().ok_or_else(|| JsValue::from_str("Node not initialized"))?;
        Ok(serde_wasm_bindgen::to_value(&node.posts)?)
    })
}

/// Récupère les infos du nœud local
#[wasm_bindgen]
pub fn get_node_info() -> Result<JsValue, JsValue> {
    NODE.with(|n| {
        let node_ref = n.borrow();
        let node = node_ref.as_ref().ok_or_else(|| JsValue::from_str("Node not initialized"))?;
        let info = serde_json::json!({
            "peer_id": node.peer_id,
            "name": node.name,
            "peers_count": node.peers.len(),
            "posts_count": node.posts.len(),
            "connected": node.ws.as_ref().map(|ws| ws.ready_state() == WebSocket::OPEN).unwrap_or(false)
        });
        Ok(serde_wasm_bindgen::to_value(&info)?)
    })
}

// ============================================
// Fonctions internes
// ============================================

fn log(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}

fn load_or_generate_identity() -> Result<(String, [u8; 32]), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let storage = window
        .local_storage()
        .map_err(|_| JsValue::from_str("localStorage error"))?
        .ok_or_else(|| JsValue::from_str("No localStorage"))?;

    // Essayer de charger la clé existante
    if let Ok(Some(key_hex)) = storage.get_item("zeta_private_key") {
        if let Ok(key_bytes) = hex::decode(&key_hex) {
            if key_bytes.len() == 32 {
                let mut private_key = [0u8; 32];
                private_key.copy_from_slice(&key_bytes);
                
                // Générer le PeerId à partir de la clé publique
                let signing_key = ed25519_dalek::SigningKey::from_bytes(&private_key);
                let public_key = signing_key.verifying_key();
                let peer_id = format!("12D3KooW{}", &hex::encode(public_key.as_bytes())[..32]);
                
                log("🔑 Clé chargée depuis localStorage");
                return Ok((peer_id, private_key));
            }
        }
    }

    // Générer une nouvelle clé
    let mut private_key = [0u8; 32];
    getrandom::getrandom(&mut private_key)
        .map_err(|e| JsValue::from_str(&format!("Random error: {}", e)))?;

    // Sauvegarder
    let key_hex = hex::encode(&private_key);
    storage
        .set_item("zeta_private_key", &key_hex)
        .map_err(|_| JsValue::from_str("Failed to save key"))?;

    // Générer le PeerId
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&private_key);
    let public_key = signing_key.verifying_key();
    let peer_id = format!("12D3KooW{}", &hex::encode(public_key.as_bytes())[..32]);

    log("🔑 Nouvelle clé générée et sauvegardée");
    Ok((peer_id, private_key))
}

fn connect_to_relay(relay_url: &str) -> Result<(), JsValue> {
    log(&format!("🔗 Connexion au relay: {}", relay_url));

    let ws = WebSocket::new(relay_url)?;
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    // Callback onopen
    let onopen = Closure::<dyn FnMut()>::new(move || {
        log("✅ Connecté au relay P2P");
        notify_status("connected");
    });
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();

    // Callback onmessage
    let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
            let text_str: String = text.into();
            handle_ws_message(&text_str);
        }
    });
    ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();

    // Callback onerror
    let onerror = Closure::<dyn FnMut()>::new(move || {
        log("❌ Erreur WebSocket");
        notify_status("error");
    });
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    // Callback onclose
    let relay_url_clone = relay_url.to_string();
    let onclose = Closure::<dyn FnMut()>::new(move || {
        log("❌ Déconnecté du relay");
        notify_status("disconnected");
        
        // Reconnexion automatique après 3 secondes
        let relay_url_inner = relay_url_clone.clone();
        let window = web_sys::window().unwrap();
        let reconnect = Closure::<dyn FnMut()>::new(move || {
            log("🔄 Tentative de reconnexion...");
            let _ = connect_to_relay(&relay_url_inner);
        });
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            reconnect.as_ref().unchecked_ref(),
            3000,
        ).unwrap();
        reconnect.forget();
    });
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    onclose.forget();

    // Sauvegarder le WebSocket
    NODE.with(|n| {
        if let Some(ref mut node) = *n.borrow_mut() {
            node.ws = Some(ws);
        }
    });

    notify_status("connecting");
    Ok(())
}

fn handle_ws_message(text: &str) {
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
        let msg_type = data.get("type").and_then(|t| t.as_str()).unwrap_or("");
        
        match msg_type {
            "init" => {
                log("📦 État initial reçu");
                
                NODE.with(|n| {
                    if let Some(ref mut node) = *n.borrow_mut() {
                        // Mettre à jour le peer_id si fourni
                        if let Some(pid) = data.get("peer_id").and_then(|p| p.as_str()) {
                            node.peer_id = pid.to_string();
                        }
                        
                        // Charger les peers
                        if let Some(peers_arr) = data.get("peers").and_then(|p| p.as_array()) {
                            node.peers.clear();
                            for peer_val in peers_arr {
                                if let Ok(peer) = serde_json::from_value::<PeerInfo>(peer_val.clone()) {
                                    node.peers.insert(peer.peer_id.clone(), peer);
                                }
                            }
                        }
                        
                        // Charger les posts
                        if let Some(posts_arr) = data.get("posts").and_then(|p| p.as_array()) {
                            node.posts.clear();
                            for post_val in posts_arr {
                                if let Ok(post) = serde_json::from_value::<Post>(post_val.clone()) {
                                    node.posts.push(post);
                                }
                            }
                        }
                    }
                });
                
                // Notifier JS
                notify_peers();
                notify_posts();
            }
            
            "new_post" | "Post" => {
                let post_data = data.get("post").or(data.get("Post")).unwrap_or(&data);
                if let Ok(post) = serde_json::from_value::<Post>(post_data.clone()) {
                    log(&format!("📨 Nouveau post de {}", post.author_name));
                    
                    NODE.with(|n| {
                        if let Some(ref mut node) = *n.borrow_mut() {
                            // Éviter les doublons
                            if !node.posts.iter().any(|p| p.id == post.id) {
                                node.posts.insert(0, post);
                            }
                        }
                    });
                    
                    notify_posts();
                }
            }
            
            "pong" => {
                // Heartbeat OK
            }
            
            _ => {
                log(&format!("📩 Message reçu: {}", msg_type));
            }
        }
    }
}

fn notify_status(status: &str) {
    NODE.with(|n| {
        let node_ref = n.borrow();
        if let Some(ref node) = *node_ref {
            if let Some(ref callback) = node.on_status {
                let _ = callback.call1(&JsValue::NULL, &JsValue::from_str(status));
            }
        }
    });
}

fn notify_peers() {
    NODE.with(|n| {
        let node_ref = n.borrow();
        if let Some(ref node) = *node_ref {
            if let Some(ref callback) = node.on_peers {
                let peers: Vec<&PeerInfo> = node.peers.values().collect();
                if let Ok(peers_js) = serde_wasm_bindgen::to_value(&peers) {
                    let _ = callback.call1(&JsValue::NULL, &peers_js);
                }
            }
        }
    });
}

fn notify_posts() {
    NODE.with(|n| {
        let node_ref = n.borrow();
        if let Some(ref node) = *node_ref {
            if let Some(ref callback) = node.on_message {
                if let Ok(posts_js) = serde_wasm_bindgen::to_value(&node.posts) {
                    let _ = callback.call1(&JsValue::NULL, &posts_js);
                }
            }
        }
    });
}

// Heartbeat périodique
#[wasm_bindgen]
pub fn start_heartbeat() {
    let window = web_sys::window().unwrap();
    
    let heartbeat = Closure::<dyn FnMut()>::new(move || {
        NODE.with(|n| {
            let node_ref = n.borrow();
            if let Some(ref node) = *node_ref {
                if let Some(ref ws) = node.ws {
                    if ws.ready_state() == WebSocket::OPEN {
                        let _ = ws.send_with_str(r#"{"type":"ping"}"#);
                    }
                }
            }
        });
    });
    
    window.set_interval_with_callback_and_timeout_and_arguments_0(
        heartbeat.as_ref().unchecked_ref(),
        30000,
    ).unwrap();
    
    heartbeat.forget();
}
