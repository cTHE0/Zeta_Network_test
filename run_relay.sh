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
    
    # Détecter si on est déjà dans le repo
    if [ -f "./Cargo.toml" ] && grep -q "zeta" "./Cargo.toml" 2>/dev/null; then
        echo -e "${GREEN}✅ Déjà dans le répertoire Zeta${NC}"
        INSTALL_DIR="$(pwd)"
    elif [ -d "$INSTALL_DIR" ]; then
        echo "Mise à jour du code existant..."
        cd "$INSTALL_DIR"
        git pull --quiet 2>/dev/null || true
    else
        echo "Clonage du repository..."
        git clone --quiet "$REPO_URL" "$INSTALL_DIR"
        cd "$INSTALL_DIR"
    fi
    
    # S'assurer que l'environnement Rust est chargé
    [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
    
    # Skip compilation si binaire existe et est récent
    if [ -f "./target/release/zeta2" ]; then
        BINARY_AGE=$(( $(date +%s) - $(stat -c %Y ./target/release/zeta2 2>/dev/null || echo 0) ))
        if [ $BINARY_AGE -lt 3600 ]; then
            echo -e "${GREEN}✅ Binaire déjà compilé (< 1h)${NC}"
            return
        fi
    fi
    
    echo -e "${YELLOW}🔨 Compilation en mode release (première fois = quelques minutes)...${NC}"
    cargo build --release 2>&1 | tail -5
    
    echo -e "${GREEN}✅ Relay compilé avec succès${NC}"
}

# ============================================================
# 6. Lancement du relay avec tunnel
# ============================================================
run_relay() {
    # Aller dans le bon répertoire
    if [ -f "./target/release/zeta2" ]; then
        INSTALL_DIR="$(pwd)"
    fi
    cd "$INSTALL_DIR"
    
    echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════╗"
    echo -e "║              🚀 DÉMARRAGE DU RELAY ZETA                     ║"
    echo -e "╚════════════════════════════════════════════════════════════╝${NC}"
    
    # Tuer les anciennes instances
    pkill -f "zeta2.*--port" 2>/dev/null || true
    pkill -f "cloudflared tunnel" 2>/dev/null || true
    sleep 1
    
    # Lancer le relay Zeta en arrière-plan
    echo -e "\n${BLUE}1️⃣ Démarrage du relay Zeta sur le port $PORT...${NC}"
    ./target/release/zeta2 --port $PORT > /tmp/zeta_relay.log 2>&1 &
    RELAY_PID=$!
    
    # Attendre que le relay soit prêt
    sleep 2
    if ! kill -0 $RELAY_PID 2>/dev/null; then
        echo -e "${RED}❌ Erreur: Le relay n'a pas pu démarrer${NC}"
        cat /tmp/zeta_relay.log
        exit 1
    fi
    echo -e "${GREEN}✅ Relay démarré (PID: $RELAY_PID)${NC}"
    
    # Lancer le tunnel Cloudflare et capturer l'URL
    echo -e "\n${BLUE}2️⃣ Création du tunnel Cloudflare...${NC}"
    
    # Utiliser un fichier pour les logs du tunnel
    TUNNEL_LOG="/tmp/cloudflared_$$.log"
    cloudflared tunnel --url http://localhost:$PORT > "$TUNNEL_LOG" 2>&1 &
    TUNNEL_PID=$!
    
    # Attendre et chercher l'URL dans les logs
    echo -e "${YELLOW}⏳ Attente de l'URL du tunnel (10-20 sec)...${NC}"
    TUNNEL_URL=""
    for i in {1..30}; do
        sleep 1
        # Chercher l'URL dans les logs
        TUNNEL_URL=$(grep -oE 'https://[a-zA-Z0-9-]+\.trycloudflare\.com' "$TUNNEL_LOG" 2>/dev/null | head -1)
        if [ -n "$TUNNEL_URL" ]; then
            break
        fi
        echo -n "."
    done
    echo ""
    
    if [ -z "$TUNNEL_URL" ]; then
        echo -e "${RED}❌ Impossible d'obtenir l'URL du tunnel${NC}"
        echo -e "${YELLOW}Logs du tunnel:${NC}"
        cat "$TUNNEL_LOG" | head -20
        echo ""
        echo -e "${YELLOW}Le relay tourne quand même en local: ws://localhost:$PORT/ws${NC}"
        echo -e "${YELLOW}Vérifiez votre connexion internet ou réessayez.${NC}"
    else
        WSS_URL="${TUNNEL_URL/https:/wss:}/ws"
        
        echo -e "\n${GREEN}╔════════════════════════════════════════════════════════════╗"
        echo -e "║                    🎉 RELAY ACTIF !                         ║"
        echo -e "╠════════════════════════════════════════════════════════════╣${NC}"
        echo -e "${GREEN}║${NC}"
        echo -e "${GREEN}║${NC}  🌐 URL HTTPS: ${CYAN}${TUNNEL_URL}${NC}"
        echo -e "${GREEN}║${NC}"
        echo -e "${GREEN}║${NC}  🔌 WebSocket: ${CYAN}${WSS_URL}${NC}"
        echo -e "${GREEN}║${NC}"
        echo -e "${GREEN}╚════════════════════════════════════════════════════════════╝${NC}"
        
        # Sauvegarder l'URL
        echo "$WSS_URL" > "$INSTALL_DIR/current_wss_url.txt"
        echo -e "\n${BLUE}💾 URL sauvegardée dans: $INSTALL_DIR/current_wss_url.txt${NC}"
        
        # Récupérer l'adresse P2P bootstrap depuis les logs
        sleep 2
        BOOTSTRAP_ADDR=$(grep "ADRESSE BOOTSTRAP" /tmp/zeta_relay.log -A 1 2>/dev/null | tail -1 | sed 's/.*   //')
        if [ -n "$BOOTSTRAP_ADDR" ]; then
            echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════╗${NC}"
            echo -e "${CYAN}║           🔗 POUR CONNECTER AVEC D'AUTRES RELAIS           ║${NC}"
            echo -e "${CYAN}╠════════════════════════════════════════════════════════════╣${NC}"
            echo -e "${CYAN}║${NC} Ajoutez cette adresse dans bootstrap.txt des autres relais:"
            echo -e "${YELLOW}$BOOTSTRAP_ADDR${NC}"
            echo -e "${CYAN}╚════════════════════════════════════════════════════════════╝${NC}"
            echo "$BOOTSTRAP_ADDR" > "$INSTALL_DIR/bootstrap_addr.txt"
        fi
        
        # Copier dans le presse-papier si possible
        if command -v xclip &> /dev/null; then
            echo "$WSS_URL" | xclip -selection clipboard
            echo -e "${GREEN}📋 URL copiée dans le presse-papier !${NC}"
        fi
    fi
    
    echo -e "\n${YELLOW}📌 Appuyez sur Ctrl+C pour arrêter le relay${NC}"
    echo ""
    
    # Fonction de nettoyage
    cleanup() {
        echo -e "\n${YELLOW}🛑 Arrêt du relay...${NC}"
        kill $RELAY_PID 2>/dev/null || true
        kill $TUNNEL_PID 2>/dev/null || true
        rm -f "$TUNNEL_LOG"
        echo -e "${GREEN}✅ Relay arrêté proprement${NC}"
        exit 0
    }
    
    trap cleanup SIGINT SIGTERM
    
    # Afficher les logs en temps réel
    echo -e "${BLUE}📊 Logs du relay:${NC}"
    tail -f /tmp/zeta_relay.log &
    TAIL_PID=$!
    
    # Garder le script actif
    wait $RELAY_PID
    kill $TAIL_PID 2>/dev/null || true
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
