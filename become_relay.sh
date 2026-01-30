#!/bin/bash
# ============================================
# Zeta Network - Script pour devenir Relais
# ============================================
#
# Ce script configure votre machine comme relais P2P.
# Cloudflare Tunnel fournit une URL WSS gratuite automatiquement.
#
# Prérequis: Linux/macOS avec accès internet
#
# Usage: ./become_relay.sh
#

set -e

echo "🌐 Zeta Network - Configuration Relais"
echo "======================================="
echo ""

# Vérifier si cloudflared est installé
if ! command -v cloudflared &> /dev/null; then
    echo "📦 Installation de Cloudflare Tunnel..."
    
    # Détecter l'OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        if command -v apt &> /dev/null; then
            curl -L https://pkg.cloudflare.com/cloudflare-main.gpg | sudo tee /usr/share/keyrings/cloudflare-archive-keyring.gpg > /dev/null
            echo "deb [signed-by=/usr/share/keyrings/cloudflare-archive-keyring.gpg] https://pkg.cloudflare.com/cloudflared focal main" | sudo tee /etc/apt/sources.list.d/cloudflared.list
            sudo apt update && sudo apt install -y cloudflared
        elif command -v yum &> /dev/null; then
            sudo yum install -y cloudflared
        else
            echo "❌ Installez cloudflared manuellement: https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation"
            exit 1
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        brew install cloudflare/cloudflare/cloudflared
    else
        echo "❌ OS non supporté. Installez cloudflared manuellement."
        exit 1
    fi
fi

echo "✅ Cloudflared installé"

# Vérifier si le binaire zeta existe
ZETA_BIN="./target/release/zeta2"
if [ ! -f "$ZETA_BIN" ]; then
    echo "📦 Compilation du relais Rust..."
    if command -v cargo &> /dev/null; then
        cargo build --release
    else
        echo "❌ Rust non installé. Installez-le: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
fi

echo "✅ Binaire zeta2 prêt"

# Port local pour le relais
LOCAL_PORT=3030

# Démarrer le relais en arrière-plan
echo ""
echo "🚀 Démarrage du relais P2P sur le port $LOCAL_PORT..."
RUST_LOG=info $ZETA_BIN --relay &
ZETA_PID=$!
sleep 3

# Vérifier que le relais tourne
if ! kill -0 $ZETA_PID 2>/dev/null; then
    echo "❌ Échec du démarrage du relais"
    exit 1
fi

echo "✅ Relais P2P démarré (PID: $ZETA_PID)"

# Démarrer Cloudflare Tunnel
echo ""
echo "🔒 Création du tunnel Cloudflare (URL WSS gratuite)..."
echo "   Ceci peut prendre quelques secondes..."
echo ""

# Le tunnel quick expose le port et donne une URL
cloudflared tunnel --url http://localhost:$LOCAL_PORT &
TUNNEL_PID=$!

# Attendre que le tunnel affiche l'URL
sleep 5

echo ""
echo "======================================="
echo "✅ RELAIS ACTIF!"
echo "======================================="
echo ""
echo "Votre relais est maintenant accessible depuis internet."
echo "L'URL Cloudflare s'affiche ci-dessus (*.trycloudflare.com)"
echo ""
echo "📋 Pour l'ajouter à la liste des relais publics:"
echo "   1. Copiez l'URL https://xxx.trycloudflare.com"
echo "   2. L'URL WebSocket sera: wss://xxx.trycloudflare.com/ws"
echo ""
echo "Appuyez sur Ctrl+C pour arrêter le relais"

# Attendre l'arrêt
trap "kill $ZETA_PID $TUNNEL_PID 2>/dev/null; echo 'Relais arrêté.'; exit 0" INT TERM

wait
