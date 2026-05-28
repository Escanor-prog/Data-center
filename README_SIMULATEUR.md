# Agent Simulateur - Data Center ENSPY

Ce projet contient le squelette Rust de l'Agent Simulateur décrit dans `repartition_simulateur-1.pdf`.

## Objectif

L'Agent Simulateur lance des attaques contrôlées pour tester les autres agents de la cellule Security Monitoring : Analyseur, Filtreur, Décideur et Auditeur.

## Structure

- `Cargo.toml` : dépendances Rust pour un serveur HTTP, l'accès réseau et le code asynchrone.
- `config/config.json` : configuration de base pour les URLs et le token d'agent.
- `src/main.rs` : point d'entrée, démarre le serveur HTTP sur le port `8005`.
- `src/control/` : état global, règles et métriques du simulateur.
- `src/api/` : routes HTTP et authentification du simulateur.
- `src/scenarios/` : catalogue des scénarios et exécuteur des attaques.
- `src/comms/` : clients de communication vers Auditeur, Décideur et Analyseur.

## Premier état (Phase 0 de démarrage)

Cette première version met en place :

- le squelette du projet Rust,
- le module de contrôle partagé,
- l'API `GET /health` et `GET /simulation/list`,
- les stubs des modules de communication et des scénarios.

## Prochaines étapes

1. Compléter `src/control/constraints.rs` et `src/control/state.rs` pour les règles de lancement.
2. Implémenter `src/scenarios/executor.rs` avec les commandes réelles (hping3, nmap, hydra, curl, etc.).
3. Compléter l'API `/simulation/start`, `/simulation/stop`, `/simulation/status` et `/metrics`.
4. Ajouter la validation mTLS et le `X-Agent-Token` dans `src/api/auth.rs`.
5. Rédiger les tests unitaires et d'intégration pour les scénarios, les routes et les communications.

## Commandes utiles

- `cargo check` : vérifier que le code compile.
- `cargo run` : démarrer l'agent.

## Notes

Le document `sec2 (2).pdf` précise les contraintes de sécurité :

- HTTPS uniquement (TLS 1.2/1.3),
- mTLS entre agents,
- `X-Agent-Token` requis,
- `/simulation/start` accessible uniquement par l'Auditeur,
- `/simulation/stop` accessible uniquement par le Décideur.
