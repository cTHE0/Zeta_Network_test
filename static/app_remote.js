// ============================================
// Zeta Network - Client Web (version pour hébergement externe)
// Se connecte au relay P2P via WebSocket
// ============================================

// État de l'application
let ws = null;
let myPeerId = null;
let myName = null;
let posts = [];
let peers = [];
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 10;

// Configuration - utilise config.js ou valeurs par défaut
const config = window.RELAY_CONFIG || {
    websocket: "ws://65.75.201.11:3030/ws",
    api: "http://65.75.201.11:3030/api"
};

// Éléments DOM
const elements = {};

// ============================================
// Initialisation
// ============================================

document.addEventListener('DOMContentLoaded', () => {
    console.log('🚀 Zeta Network - Démarrage...');
    console.log('📡 Relay configuré:', config.websocket);
    
    // Récupérer les éléments DOM
    elements.connectionIndicator = document.getElementById('connection-indicator');
    elements.connectionText = document.getElementById('connection-text');
    elements.localPeerId = document.getElementById('local-peer-id');
    elements.localName = document.getElementById('local-name');
    elements.nodeMode = document.getElementById('node-mode');
    elements.peerCount = document.getElementById('peer-count');
    elements.peersList = document.getElementById('peers-list');
    elements.postsFeed = document.getElementById('posts-feed');
    elements.postCountBadge = document.getElementById('post-count-badge');
    elements.postForm = document.getElementById('post-form');
    elements.authorInput = document.getElementById('author');
    elements.contentInput = document.getElementById('content');
    elements.charCount = document.getElementById('char-count');
    elements.submitBtn = document.getElementById('submit-btn');

    // Charger le nom d'utilisateur sauvegardé
    const savedUsername = localStorage.getItem('zeta_username');
    if (savedUsername && elements.authorInput) {
        elements.authorInput.value = savedUsername;
    }

    // Gestionnaires d'événements
    if (elements.postForm) {
        elements.postForm.addEventListener('submit', handleSubmit);
    }
    if (elements.contentInput) {
        elements.contentInput.addEventListener('input', updateCharCount);
    }

    // Connexion WebSocket au relay
    connectWebSocket();
});

// ============================================
// WebSocket - Connexion au relay distant
// ============================================

function connectWebSocket() {
    console.log('🔌 Connexion au relay:', config.websocket);
    updateConnectionStatus('connecting');
    
    try {
        ws = new WebSocket(config.websocket);
        
        ws.onopen = () => {
            console.log('✅ Connecté au réseau P2P via le relay');
            reconnectAttempts = 0;
            updateConnectionStatus('connected');
        };
        
        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                handleWebSocketMessage(data);
            } catch (e) {
                console.error('Erreur parsing message:', e);
            }
        };
        
        ws.onclose = (event) => {
            console.log('❌ Déconnecté du relay', event.code, event.reason);
            updateConnectionStatus('disconnected');
            scheduleReconnect();
        };
        
        ws.onerror = (error) => {
            console.error('Erreur WebSocket:', error);
            updateConnectionStatus('error');
        };
    } catch (e) {
        console.error('Erreur création WebSocket:', e);
        updateConnectionStatus('error');
        scheduleReconnect();
    }
}

function scheduleReconnect() {
    if (reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
        reconnectAttempts++;
        const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000);
        console.log(`🔄 Reconnexion dans ${delay/1000}s (tentative ${reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`);
        setTimeout(connectWebSocket, delay);
    } else {
        console.error('❌ Nombre maximum de tentatives atteint');
        updateConnectionStatus('error');
        if (elements.connectionText) {
            elements.connectionText.textContent = 'Impossible de se connecter au relay';
        }
    }
}

// ============================================
// Gestion des messages WebSocket
// ============================================

function handleWebSocketMessage(data) {
    console.log('📩 Message:', data.type);
    
    switch (data.type) {
        case 'init':
            // État initial reçu du relay
            myPeerId = data.peer_id;
            peers = data.peers || [];
            posts = data.posts || [];
            console.log('🆔 Mon PeerId:', myPeerId);
            console.log('👥 Peers:', peers.length, '| 📝 Posts:', posts.length);
            updateUI();
            break;
            
        case 'new_post':
        case 'Post':
            // Nouveau post reçu via GossipSub
            const post = data.post || data.Post || data;
            if (post && post.id) {
                // Éviter les doublons
                if (!posts.find(p => p.id === post.id)) {
                    posts.unshift(post);
                    updatePostsFeed();
                    
                    // Notification si ce n'est pas notre propre post
                    if (post.author !== myPeerId) {
                        showNotification(`📨 ${post.author_name}: ${post.content.substring(0, 50)}...`);
                    }
                }
            }
            break;
            
        case 'peer_joined':
        case 'PeerJoined':
            console.log('👋 Nouveau peer:', data.peer_id || data.PeerJoined?.peer_id);
            // Rafraîchir via REST ou attendre le prochain message
            break;
            
        case 'peer_left':
        case 'PeerLeft':
            console.log('👋 Peer parti:', data.peer_id || data.PeerLeft?.peer_id);
            const leftId = data.peer_id || data.PeerLeft?.peer_id;
            if (leftId) {
                peers = peers.filter(p => p.peer_id !== leftId);
                updatePeersList();
            }
            break;
            
        case 'pong':
            // Réponse au ping, connexion active
            break;
            
        default:
            console.log('Message inconnu:', data);
    }
}

// ============================================
// Envoi de posts
// ============================================

async function handleSubmit(e) {
    e.preventDefault();
    
    const author = elements.authorInput?.value?.trim();
    const content = elements.contentInput?.value?.trim();
    
    if (!author || !content) {
        alert('Veuillez remplir tous les champs');
        return;
    }
    
    // Sauvegarder le nom
    localStorage.setItem('zeta_username', author);
    
    // Désactiver le bouton
    if (elements.submitBtn) {
        elements.submitBtn.disabled = true;
        elements.submitBtn.textContent = '📤 Publication...';
    }
    
    try {
        // Envoyer via WebSocket
        if (ws && ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({
                type: 'post',
                content: content,
                author_name: author
            }));
            
            console.log('📤 Post envoyé via WebSocket');
            
            // Vider le champ
            if (elements.contentInput) {
                elements.contentInput.value = '';
            }
            updateCharCount();
            
        } else {
            // Fallback: envoyer via REST API
            await sendPostREST(author, content);
        }
        
    } catch (error) {
        console.error('❌ Erreur envoi:', error);
        alert('Erreur lors de l\'envoi: ' + error.message);
    } finally {
        if (elements.submitBtn) {
            elements.submitBtn.disabled = false;
            elements.submitBtn.textContent = '📤 Publier';
        }
    }
}

async function sendPostREST(author, content) {
    const response = await fetch(`${config.api}/post`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            author_name: author,
            content: content
        })
    });
    
    if (!response.ok) {
        throw new Error('Erreur serveur: ' + response.status);
    }
    
    const post = await response.json();
    console.log('📤 Post envoyé via REST:', post);
    
    // Ajouter localement
    if (!posts.find(p => p.id === post.id)) {
        posts.unshift(post);
        updatePostsFeed();
    }
    
    // Vider le champ
    if (elements.contentInput) {
        elements.contentInput.value = '';
    }
    updateCharCount();
}

// ============================================
// Mise à jour de l'interface
// ============================================

function updateUI() {
    updateLocalInfo();
    updatePeersList();
    updatePostsFeed();
}

function updateLocalInfo() {
    if (elements.localPeerId && myPeerId) {
        elements.localPeerId.textContent = myPeerId.substring(0, 20) + '...';
        elements.localPeerId.title = myPeerId;
    }
    
    if (elements.localName) {
        elements.localName.textContent = myName || `Browser-${myPeerId?.substring(8, 16) || 'Unknown'}`;
    }
    
    if (elements.nodeMode) {
        elements.nodeMode.innerHTML = '<span class="badge">🌐 WebSocket Client</span>';
    }
}

function updatePeersList() {
    if (elements.peerCount) {
        elements.peerCount.textContent = peers.length;
    }
    
    if (!elements.peersList) return;
    
    if (peers.length === 0) {
        elements.peersList.innerHTML = '<p class="empty-state">Connexion au réseau...</p>';
        return;
    }
    
    elements.peersList.innerHTML = peers.map(peer => `
        <div class="peer-item ${peer.is_browser ? 'browser-peer' : 'native-peer'}">
            <span class="peer-icon">${peer.is_browser ? '🌐' : '💻'}</span>
            <div class="peer-info">
                <span class="peer-name">${escapeHtml(peer.name || 'Anonyme')}</span>
                <span class="peer-id">${peer.peer_id.substring(0, 16)}...</span>
            </div>
            <span class="peer-type badge">${peer.is_browser ? 'Browser' : 'Natif'}</span>
        </div>
    `).join('');
}

function updatePostsFeed() {
    if (elements.postCountBadge) {
        elements.postCountBadge.textContent = posts.length;
    }
    
    if (!elements.postsFeed) return;
    
    if (posts.length === 0) {
        elements.postsFeed.innerHTML = '<p class="empty-state">Aucun post. Soyez le premier à publier !</p>';
        return;
    }
    
    elements.postsFeed.innerHTML = posts.map(post => {
        const date = new Date(post.timestamp * 1000);
        const timeAgo = getTimeAgo(date);
        const isLocal = post.author === myPeerId;
        
        return `
            <div class="post ${isLocal ? 'post-local' : ''}" data-id="${post.id}">
                <div class="post-header">
                    <span class="post-author">${escapeHtml(post.author_name)}</span>
                    <span class="post-time" title="${date.toLocaleString()}">${timeAgo}</span>
                </div>
                <div class="post-content">${escapeHtml(post.content)}</div>
                <div class="post-footer">
                    <span class="post-peer-id" title="${post.author}">
                        ${isLocal ? '📍 Vous' : `🔗 ${post.author.substring(0, 12)}...`}
                    </span>
                </div>
            </div>
        `;
    }).join('');
}

function updateConnectionStatus(status) {
    if (elements.connectionIndicator) {
        elements.connectionIndicator.className = `status-indicator status-${status}`;
    }
    
    if (elements.connectionText) {
        const statusTexts = {
            'connecting': '🔄 Connexion au relay...',
            'connected': '✅ Connecté au réseau P2P',
            'disconnected': '❌ Déconnecté',
            'error': '⚠️ Erreur de connexion'
        };
        elements.connectionText.textContent = statusTexts[status] || status;
    }
}

function updateCharCount() {
    if (elements.charCount && elements.contentInput) {
        elements.charCount.textContent = elements.contentInput.value.length;
    }
}

// ============================================
// Utilitaires
// ============================================

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function getTimeAgo(date) {
    const seconds = Math.floor((new Date() - date) / 1000);
    
    if (seconds < 60) return 'À l\'instant';
    if (seconds < 3600) return `Il y a ${Math.floor(seconds / 60)} min`;
    if (seconds < 86400) return `Il y a ${Math.floor(seconds / 3600)}h`;
    return date.toLocaleDateString('fr-FR');
}

function showNotification(message) {
    // Notification simple via console pour l'instant
    console.log('🔔', message);
    
    // On pourrait ajouter une notification visuelle ici
    // ou utiliser l'API Notification si autorisée
}

// ============================================
// Heartbeat pour maintenir la connexion
// ============================================

setInterval(() => {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'ping' }));
    }
}, 30000);

console.log('📦 Zeta Network client chargé');
console.log('🔗 Relay:', config.websocket);
