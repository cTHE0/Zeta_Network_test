#!/bin/bash

# Script de démarrage pour Zeta2

echo "🌐 Zeta2 - Réseau Social Décentralisé"
echo "======================================"
echo ""

# Vérifier si Rust est installé
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo n'est pas installé. Installez Rust depuis https://rustup.rs/"
    exit 1
fi

# Menu de sélection
echo "Sélectionnez le mode de démarrage :"
echo "1) Mode RELAY (serveur public)"
echo "2) Mode CLIENT (derrière NAT avec relay)"
echo "3) Mode LOCAL (réseau local sans relay)"
echo ""
read -p "Votre choix (1-3) : " choice

case $choice in
    1)
        echo ""
        echo "🖥️  Démarrage en mode RELAY..."
        echo "Le serveur écoutera sur le port 4001"
        echo ""
        cargo run --release -- --relay
        ;;
    2)
        echo ""
        read -p "Adresse du relay (ex: /ip4/192.168.1.100/tcp/4001/p2p/12D3...) : " relay_addr
        if [ -z "$relay_addr" ]; then
            echo "❌ Adresse relay requise"
            exit 1
        fi
        echo ""
        echo "💻 Démarrage en mode CLIENT..."
        echo "Connexion au relay: $relay_addr"
        echo ""
        cargo run --release -- --relay-addr "$relay_addr"
        ;;
    3)
        echo ""
        echo "🏠 Démarrage en mode LOCAL..."
        echo "Découverte automatique via mDNS activée"
        echo ""
        cargo run --release
        ;;
    *)
        echo "❌ Choix invalide"
        exit 1
        ;;
esac
