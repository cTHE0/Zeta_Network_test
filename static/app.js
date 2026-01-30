// ============================================
// Zeta2 - Application Web avec WebSocket
// ============================================

// État de l'application
let ws = null;
let myPeerId = null;
let posts = [];
let peers = [];
let reconnectAttempts = 0;
const MAX_RECONNECT_ATTEMPTS = 10;

// Éléments DOM
const elements = {
    connectionIndicator: null,
    connectionText: null,
    localPeerId: null,
    localName: null,
    nodeMode: null,
    peerCount: null,
    peersList: null,
    postsFeed: null,
    postCountBadge: null,
    postForm: null,
    authorInput: null,
    contentInput: null,
    charCount: null,
    submitBtn: null
};

// Initialisation
document.addEventListener('DOMContentLoaded', () => {
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
    const savedUsername = localStorage.getItem('zeta2_username');
    if (savedUsername) {
        elements.authorInput.value = savedUsername;
    }

    // Gestionnaires d'événements
    elements.postForm.addEventListener('submit', handleSubmit);
    elements.contentInput.addEventListener('input', updateCharCount);

    // Connexion WebSocket
    connectWebSocket();
    
    // Fallback: récupérer via REST si WebSocket échoue
    fetchNetworkInfoREST();
});

// ============================================
// WebSocket
// ============================================

function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${protocol}//${window.location.host}/ws`;
    
    console.log('🔌 Connexion WebSocket:', wsUrl);
    updateConnectionStatus('connecting');
    
    try {
        ws = new WebSocket(wsUrl);
        
        ws.onopen = () => {
            console.log('✅ WebSocket connecté');
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
        
        ws.onclose = () => {
            console.log('❌ WebSocket déconnecté');
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
    }
}

function handleWebSocketMessage(data) {
    console.log('📩 Message WS:', data.type);
    
    switch (data.type) {
        case 'init':
            // État initial reçu
            myPeerId = data.peer_id;
            peers = data.peers || [];
            posts = data.posts || [];
            updateUI();
            break;
            
        case 'new_post':
            // Nouveau post reçu
            if (data.post) {
                // Éviter les doublons
                if (!posts.find(p => p.id === data.post.id)) {
                    posts.unshift(data.post);
                    updatePostsFeed();
                    showNotification(`📨 Nouveau post de ${data.post.author_name}`);
                }
            }
            break;
            
        case 'peer_joined':
            console.log('👋 Peer rejoint:', data.peer_id);
            fetchNetworkInfoREST(); // Rafraîchir la liste complète
            break;
            
        case 'peer_left':
            console.log('👋 Peer parti:', data.peer_id);
            peers = peers.filter(p => p.peer_id !== data.peer_id);
            updatePeersList();
            break;
            
        case 'pong':
            // Réponse au ping
            break;
            
        default:
            console.log('Message inconnu:', data);
    }
}

function sendWebSocketMessage(message) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(message));
        return true;
    }
    return false;
}

// ============================================
// REST API (fallback)
// ============================================

async function fetchNetworkInfoREST() {
    try {
        const response = await fetch('/api/network');
        if (!response.ok) throw new Error('Erreur réseau');
        
        const data = await response.json();
        
        // Mettre à jour l'état
        elements.localPeerId.textContent = truncateId(data.local_peer_id);
        elements.localName.textContent = data.local_name || '-';
        elements.nodeMode.textContent = data.is_relay ? '🖥️ Relay' : '💻 Client';
        
        // Si pas de WebSocket, utiliser ces données
        if (!ws || ws.readyState !== WebSocket.OPEN) {
            peers = data.peers || [];
            posts = data.posts || [];
            updateUI();
        }
    } catch (error) {
        console.error('Erreur REST:', error);
    }
}

// ============================================
// Interface utilisateur
// ============================================

function updateConnectionStatus(status) {
    const indicator = elements.connectionIndicator;
    const text = elements.connectionText;
    
    indicator.className = 'status-indicator';
    
    switch (status) {
        case 'connected':
            indicator.classList.add('status-connected');
            text.textContent = 'Connecté (WebSocket)';
            break;
        case 'connecting':
            indicator.classList.add('status-connecting');
            text.textContent = 'Connexion...';
            break;
        case 'disconnected':
            indicator.classList.add('status-disconnected');
            text.textContent = 'Déconnecté';
            break;
        case 'error':
            indicator.classList.add('status-error');
            text.textContent = 'Erreur';
            break;
    }
}

function updateUI() {
    updatePeersList();
    updatePostsFeed();
    elements.peerCount.textContent = peers.length;
    elements.postCountBadge.textContent = posts.length;
}

function updatePeersList() {
    if (peers.length === 0) {
        elements.peersList.innerHTML = '<p class="empty-state">Aucun pair connecté</p>';
        return;
    }

    const html = peers.map(peer => {
        const icon = peer.is_browser ? '🌐' : '💻';
        const name = peer.name || 'Anonyme';
        return `
            <div class="peer-item">
                <div class="peer-id">
                    <span class="status-indicator status-connected"></span>
                    ${icon} ${truncateId(peer.peer_id)}
                </div>
                <div class="peer-info">
                    <span class="peer-name">${escapeHtml(name)}</span>
                    <span class="peer-address">${peer.address || '-'}</span>
                </div>
            </div>
        `;
    }).join('');

    elements.peersList.innerHTML = html;
    elements.peerCount.textContent = peers.length;
}

function updatePostsFeed() {
    if (posts.length === 0) {
        elements.postsFeed.innerHTML = '<p class="empty-state">Aucun post pour le moment</p>';
        return;
    }

    const html = posts.map(post => `
        <div class="post-item">
            <div class="post-header">
                <span class="post-author">👤 ${escapeHtml(post.author_name || post.author)}</span>
                <span class="post-time">${formatTimestamp(post.timestamp)}</span>
            </div>
            <div class="post-content">${escapeHtml(post.content)}</div>
            <div class="post-meta">
                <span class="post-id">🔗 ${truncateId(post.author)}</span>
            </div>
        </div>
    `).join('');

    elements.postsFeed.innerHTML = html;
    elements.postCountBadge.textContent = posts.length;
}

function updateCharCount() {
    const count = elements.contentInput.value.length;
    elements.charCount.textContent = count;
    elements.charCount.style.color = count > 250 ? '#e0245e' : '#8899a6';
}

// ============================================
// Actions
// ============================================

async function handleSubmit(event) {
    event.preventDefault();
    
    const authorName = elements.authorInput.value.trim();
    const content = elements.contentInput.value.trim();
    
    if (!authorName || !content) {
        showNotification('❌ Veuillez remplir tous les champs', true);
        return;
    }

    // Sauvegarder le nom
    localStorage.setItem('zeta2_username', authorName);

    // Désactiver le bouton
    elements.submitBtn.disabled = true;
    elements.submitBtn.textContent = '⏳ Publication...';

    // Essayer d'envoyer via WebSocket d'abord
    const wsSent = sendWebSocketMessage({
        type: 'post',
        content: content,
        author_name: authorName
    });

    if (wsSent) {
        // WebSocket OK
        elements.contentInput.value = '';
        elements.charCount.textContent = '0';
        showNotification('✅ Post publié via WebSocket !');
    } else {
        // Fallback REST API
        try {
            const response = await fetch('/api/post', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ content, author_name: authorName })
            });

            if (!response.ok) throw new Error('Erreur publication');

            elements.contentInput.value = '';
            elements.charCount.textContent = '0';
            showNotification('✅ Post publié via REST !');
            fetchNetworkInfoREST();
        } catch (error) {
            console.error('Erreur:', error);
            showNotification('❌ Erreur lors de la publication', true);
        }
    }

    // Réactiver le bouton
    elements.submitBtn.disabled = false;
    elements.submitBtn.textContent = '📤 Publier';
}

// ============================================
// Utilitaires
// ============================================

function truncateId(id) {
    if (!id) return '-';
    if (id.length <= 16) return id;
    return id.substring(0, 8) + '...' + id.substring(id.length - 6);
}

function formatTimestamp(timestamp) {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = Math.floor((now - date) / 1000);

    if (diff < 60) return 'À l\'instant';
    if (diff < 3600) return `Il y a ${Math.floor(diff / 60)} min`;
    if (diff < 86400) return `Il y a ${Math.floor(diff / 3600)} h`;
    
    return date.toLocaleDateString('fr-FR', {
        day: 'numeric',
        month: 'short',
        hour: '2-digit',
        minute: '2-digit'
    });
}

function escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

function showNotification(message, isError = false) {
    console.log(message);
    // Simple notification visuelle (pourrait être amélioré)
    const notification = document.createElement('div');
    notification.className = `notification ${isError ? 'error' : 'success'}`;
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 15px 20px;
        border-radius: 8px;
        background: ${isError ? '#e0245e' : '#17bf63'};
        color: white;
        z-index: 1000;
        animation: fadeIn 0.3s ease;
    `;
    document.body.appendChild(notification);
    
    setTimeout(() => {
        notification.style.opacity = '0';
        setTimeout(() => notification.remove(), 300);
    }, 3000);
}

// Ping périodique pour maintenir la connexion
setInterval(() => {
    if (ws && ws.readyState === WebSocket.OPEN) {
        sendWebSocketMessage({ type: 'ping' });
    }
}, 30000);

// Rafraîchir périodiquement via REST (backup)
setInterval(fetchNetworkInfoREST, 10000);
