// ============================================
// Zeta Network - Configuration
// ============================================

// Adresse du relay P2P (serveur ServerCheap)
// Modifier cette valeur pour pointer vers votre relay
const RELAY_CONFIG = {
    // WebSocket du relay pour les clients navigateur
    websocket: "ws://65.75.201.11:3030/ws",
    
    // API REST du relay
    api: "http://65.75.201.11:3030/api",
    
    // Adresse P2P du relay (pour les clients natifs)
    p2p: "/ip4/65.75.201.11/tcp/4001/p2p/12D3KooWCqv678TveF9HXgxV1gvMtLgCHS5bM8qDfXyfHPEdKqhS"
};

// Exporter pour utilisation dans d'autres scripts
if (typeof window !== 'undefined') {
    window.RELAY_CONFIG = RELAY_CONFIG;
}

if (typeof module !== 'undefined' && module.exports) {
    module.exports = RELAY_CONFIG;
}
