#!/bin/bash
# ============================================================
# 🚀 ZETA RELAY - Script tout-en-un
# Une seule commande pour devenir un relais du réseau Zeta
# ============================================================
# Usage: curl -sSL https://raw.githubusercontent.com/cTHE0/Zeta_Network_test/main/run_relay.sh | bash
# Ou:    ./run_relay.sh
# ============================================================

set -e

PORT=${ZETA_PORT:-3030}
REPO_URL="https://github.com/cTHE0/Zeta_Network_test.git"
INSTALL_DIR="${ZETA_DIR:-$HOME/zeta-relay}"

# Couleurs
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║         🌐 ZETA NETWORK - Relay Setup Script               ║"
echo "║         Réseau Social Décentralisé P2P                     ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# ============================================================
# 1. Détection OS et architecture
# ============================================================
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if [ -f /etc/debian_version ]; then
            echo "debian"
        elif [ -f /etc/redhat-release ]; then
            echo "redhat"
        elif [ -f /etc/arch-release ]; then
            echo "arch"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

detect_arch() {
    case $(uname -m) in
        x86_64)  echo "amd64" ;;
        aarch64) echo "arm64" ;;
        armv7l)  echo "arm" ;;
        *)       echo "amd64" ;;
    esac
}

OS=$(detect_os)
ARCH=$(detect_arch)
echo -e "${BLUE}📍 Système détecté: $OS ($ARCH)${NC}"

# ============================================================
# 2. Installation des dépendances
# ============================================================
install_deps() {
    echo -e "\n${YELLOW}📦 Installation des dépendances...${NC}"
    
    case $OS in
        debian)
            sudo apt-get update -qq
            sudo apt-get install -y -qq curl git build-essential pkg-config libssl-dev
            ;;
        redhat)
            sudo yum install -y curl git gcc openssl-devel
            ;;
        arch)
            sudo pacman -Sy --noconfirm curl git base-devel openssl
            ;;
        macos)
            if ! command -v brew &> /dev/null; then
                echo "Installation de Homebrew..."
                /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            fi
            brew install curl git openssl
            ;;
    esac
}

# ============================================================
# 3. Installation de Rust
# ============================================================
install_rust() {
    if command -v cargo &> /dev/null; then
        echo -e "${GREEN}✅ Rust déjà installé: $(rustc --version)${NC}"
        return
    fi
    
    echo -e "\n${YELLOW}🦀 Installation de Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✅ Rust installé: $(rustc --version)${NC}"
}

# ============================================================
# 4. Installation de Cloudflared
# ============================================================
install_cloudflared() {
    if command -v cloudflared &> /dev/null; then
        echo -e "${GREEN}✅ Cloudflared déjà installé${NC}"
        return
    fi
    
    echo -e "\n${YELLOW}☁️ Installation de Cloudflared...${NC}"
    
    case $OS in
        debian)
            # Méthode officielle Cloudflare
            curl -L https://pkg.cloudflare.com/cloudflare-main.gpg | sudo tee /usr/share/keyrings/cloudflare-archive-keyring.gpg > /dev/null
            echo "deb [signed-by=/usr/share/keyrings/cloudflare-archive-keyring.gpg] https://pkg.cloudflare.com/cloudflared $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/cloudflared.list
            sudo apt-get update -qq && sudo apt-get install -y -qq cloudflared 2>/dev/null || {
                # Fallback: téléchargement direct
                echo "Fallback: téléchargement direct..."
                curl -L "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-$ARCH" -o /tmp/cloudflared
                chmod +x /tmp/cloudflared
                sudo mv /tmp/cloudflared /usr/local/bin/cloudflared
            }
            ;;
        redhat)
            curl -L "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-$ARCH" -o /tmp/cloudflared
            chmod +x /tmp/cloudflared
            sudo mv /tmp/cloudflared /usr/local/bin/cloudflared
            ;;
        arch)
            yay -S --noconfirm cloudflared 2>/dev/null || {
                curl -L "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-$ARCH" -o /tmp/cloudflared
                chmod +x /tmp/cloudflared
                sudo mv /tmp/cloudflared /usr/local/bin/cloudflared
            }
            ;;
        macos)
            brew install cloudflared
            ;;
        *)
            curl -L "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-$ARCH" -o /tmp/cloudflared
            chmod +x /tmp/cloudflared
            sudo mv /tmp/cloudflared /usr/local/bin/cloudflared
            ;;
    esac
    
    echo -e "${GREEN}✅ Cloudflared installé${NC}"
}

# ============================================================
# 5. Clonage et compilation du relay
# ============================================================
build_relay() {
    echo -e "\n${YELLOW}📥 Préparation du relay Zeta...${NC}"
    
    if [ -d "$INSTALL_DIR" ]; then
        echo "Mise à jour du code existant..."
        cd "$INSTALL_DIR"
        git pull --quiet
    else
        echo "Clonage du repository..."
        git clone --quiet "$REPO_URL" "$INSTALL_DIR"
        cd "$INSTALL_DIR"
    fi
    
    # S'assurer que l'environnement Rust est chargé
    [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
    
    echo -e "${YELLOW}🔨 Compilation en mode release (peut prendre quelques minutes)...${NC}"
    cargo build --release --quiet
    
    echo -e "${GREEN}✅ Relay compilé avec succès${NC}"
}

# ============================================================
# 6. Lancement du relay avec tunnel
# ============================================================
run_relay() {
    cd "$INSTALL_DIR"
    
    echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════╗"
    echo -e "║              🚀 DÉMARRAGE DU RELAY ZETA                     ║"
    echo -e "╚════════════════════════════════════════════════════════════╝${NC}"
    
    # Créer un fichier temporaire pour capturer l'URL du tunnel
    TUNNEL_URL_FILE=$(mktemp)
    
    # Lancer le relay Zeta en arrière-plan
    echo -e "\n${BLUE}1️⃣ Démarrage du relay Zeta sur le port $PORT...${NC}"
    ./target/release/zeta2 --port $PORT &
    RELAY_PID=$!
    
    # Attendre que le relay soit prêt
    sleep 3
    if ! kill -0 $RELAY_PID 2>/dev/null; then
        echo -e "${RED}❌ Erreur: Le relay n'a pas pu démarrer${NC}"
        exit 1
    fi
    echo -e "${GREEN}✅ Relay démarré (PID: $RELAY_PID)${NC}"
    
    # Lancer le tunnel Cloudflare
    echo -e "\n${BLUE}2️⃣ Création du tunnel Cloudflare...${NC}"
    cloudflared tunnel --url http://localhost:$PORT 2>&1 | tee /dev/stderr | grep -oP 'https://[a-z0-9-]+\.trycloudflare\.com' | head -1 > "$TUNNEL_URL_FILE" &
    TUNNEL_PID=$!
    
    # Attendre l'URL du tunnel
    echo -e "${YELLOW}⏳ Attente de l'URL du tunnel...${NC}"
    for i in {1..30}; do
        if [ -s "$TUNNEL_URL_FILE" ]; then
            TUNNEL_URL=$(cat "$TUNNEL_URL_FILE")
            break
        fi
        sleep 1
    done
    
    if [ -z "$TUNNEL_URL" ]; then
        echo -e "${RED}❌ Impossible d'obtenir l'URL du tunnel${NC}"
        echo -e "${YELLOW}Le relay tourne quand même en local sur le port $PORT${NC}"
    else
        WSS_URL="${TUNNEL_URL/https:/wss:}/ws"
        
        echo -e "\n${GREEN}╔════════════════════════════════════════════════════════════╗"
        echo -e "║                    🎉 RELAY ACTIF !                         ║"
        echo -e "╠════════════════════════════════════════════════════════════╣"
        echo -e "║                                                            ║"
        echo -e "║  🌐 URL HTTPS: ${TUNNEL_URL}"
        echo -e "║                                                            ║"
        echo -e "║  🔌 WebSocket WSS: ${WSS_URL}"
        echo -e "║                                                            ║"
        echo -e "║  📋 Copiez l'URL WSS dans la config du site Zeta           ║"
        echo -e "║                                                            ║"
        echo -e "╚════════════════════════════════════════════════════════════╝${NC}"
        
        # Sauvegarder l'URL dans un fichier
        echo "$WSS_URL" > "$INSTALL_DIR/current_wss_url.txt"
        echo -e "\n${BLUE}💾 URL sauvegardée dans: $INSTALL_DIR/current_wss_url.txt${NC}"
    fi
    
    echo -e "\n${YELLOW}📌 Appuyez sur Ctrl+C pour arrêter le relay${NC}"
    echo ""
    
    # Fonction de nettoyage
    cleanup() {
        echo -e "\n${YELLOW}🛑 Arrêt du relay...${NC}"
        kill $RELAY_PID 2>/dev/null || true
        kill $TUNNEL_PID 2>/dev/null || true
        rm -f "$TUNNEL_URL_FILE"
        echo -e "${GREEN}✅ Relay arrêté proprement${NC}"
        exit 0
    }
    
    trap cleanup SIGINT SIGTERM
    
    # Garder le script actif
    wait $RELAY_PID
}

# ============================================================
# MAIN
# ============================================================
main() {
    # Vérifier si on a besoin d'installer des dépendances
    if ! command -v git &> /dev/null || ! command -v curl &> /dev/null; then
        install_deps
    fi
    
    install_rust
    install_cloudflared
    build_relay
    run_relay
}

main "$@"
