#!/bin/bash
# ============================================================
# ZETA RELAY - Setup Cloudflare Tunnel pour WSS
# À exécuter sur le serveur ServerCheap (65.75.201.11)
# ============================================================

set -e

echo "🔧 Installation de cloudflared..."

# Télécharger cloudflared si pas présent
if ! command -v cloudflared &> /dev/null; then
    curl -L https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /tmp/cloudflared
    chmod +x /tmp/cloudflared
    sudo mv /tmp/cloudflared /usr/local/bin/cloudflared
fi

echo "✅ cloudflared installé: $(cloudflared --version)"

# Vérifier que le relay tourne
if ! curl -s http://localhost:3030/api/network > /dev/null 2>&1; then
    echo "⚠️ Le relay ne tourne pas sur le port 3030!"
    echo "   Lancez d'abord: cargo run --release"
    exit 1
fi

echo "🌐 Démarrage du tunnel Cloudflare..."
echo "   Votre URL WSS apparaîtra ci-dessous (format: xxx.trycloudflare.com)"
echo ""
echo "============================================================"

# Lancer le tunnel (affiche l'URL automatiquement)
cloudflared tunnel --url http://localhost:3030

# Note: L'URL sera affichée dans les logs, genre:
# "Your quick Tunnel has been created! Visit it at (it may take some time to be reachable):
# https://xxx-xxx-xxx.trycloudflare.com"
