# Agent Simulateur — GANDAL (Rust)

Agent de simulation d'attaques contrôlées pour la cellule Security Monitoring ENSPY (~150 personnes).

## Pipeline complet (`POST /simulation/start`)

À chaque lancement, **tous les agents** sont sollicités :

| Étape | Agent | Rôle |
|-------|--------|------|
| Validation | Simulateur | Contraintes **C1–C15** |
| Handshake | **Décideur** | `simulation_token`, mode simulation (C5) |
| Phase 1 | **Filtreur** | Trafic Internet simulé (`phase1_scenario`) |
| Phase 2 | **Analyseur** | Trafic interne sandbox (`SIM_START` / `SIM_END`) |
| Phase 3 | **Chiffreur** | Mot de passe faible + encrypt/decrypt + `cert_tls` |
| Intégrité | **Auditeur** | Events `log_event` + test `injection_base` |
| Fin | **Décideur** | `POST /simulation/phase_ack` ou auto-ACK en `dry_run` |

## Base de connaissances — 25 scénarios

Fichier : [`knowledge/attacks_catalog.json`](knowledge/attacks_catalog.json) (MITRE ATT&CK + anomalies ANO-xxx Phase 0).

| Agent | Nb | Exemples |
|-------|-----|----------|
| Filtreur | 7 | `ddos_syn`, `scan_ports`, `brute_force_externe`, `ping_flood`, `port_interdit`, `contact_cc`, `exfiltration_externe` |
| Analyseur | 14 | `syn_flood_interne`, `brute_force`, `tunnel_dns`, `mitm`, `ransomware_sim`, `panne_reseau`, … |
| Chiffreur | 3 | `cert_tls`, `mot_de_passe_faible`, `rotation_cle` |
| Auditeur | 1 | `injection_base` |

Liste complète : `GET /simulation/list` (inclut `mitre_tactic`, timeouts par scénario).

## Timeouts recommandés (fixés dans le catalogue)

| Paramètre | Valeur | Raison |
|-----------|--------|--------|
| Détection phase 1 (Filtreur) | **60 s** | Périmètre Internet, réaction Filtreur rapide |
| Détection phase 2 (Analyseur) | **90 s** | Mini-proxies + corrélation inter-VM |
| Détection phase 3 (Chiffreur) | **45 s** | API REST, pas de flood réseau |
| Exécution DDoS / scan | **120–180 s** | Plafond charge sandbox |
| Brute force | **300 s** | Hydra réaliste mais borné |
| Crypto / API | **60–90 s** | Tests credential |
| **Pipeline global (C14)** | **900 s (15 min)** | Arrêt auto si débordement |

Le timeout effectif par phase = `max(detection_timeout_scénario, detection_phaseN)` depuis `timeouts_policy` du JSON.

## Contraintes C1–C15 (binôme 11)

| ID | Implémentation |
|----|----------------|
| C1 | Cible dans `sandbox_cidr` |
| C2 | `authorized_by` (+ `authorization_ref` si `require_authorization_ref`) |
| C3 | `X-Agent-Token` valide |
| C4 | `snapshot_id` si `require_snapshot` (prod) |
| C5 | Handshake Décideur ; `SIM_START` Analyseur (fallback direct) |
| C6 | Event `C6_CELLULES_NOTIFIEES` → Auditeur |
| C7 | `GET /health` Décideur + Analyseur si `enforce_health_checks` |
| C8 | Exécution limitée aux VMs sandbox (config + catalogue) |
| C9 | Une seule simulation active |
| C10 | Fenêtre horaire + `max_duration` ≤ plafond scénario |
| C11 | Interdit cibles dans `production_cidrs` |
| C12 | `rollback_plan_id` si `require_rollback_plan` (prod) |
| C13 | Journal `simulation_id` isolé (events Auditeur + backup local) |
| C14 | `global_max_duration_secs` + `POST /simulation/stop` |
| C15 | Health check post-pipeline |

En **dry_run** (défaut) : C4/C7/C12 assouplies ; détection simulée en ~2 s.

## Endpoints

| Route | Méthode | Appelant |
|-------|---------|----------|
| `/health` | GET | tous |
| `/metrics` | GET | Auditeur |
| `/simulation/list` | GET | Auditeur |
| `/simulation/status` | GET | Auditeur |
| `/simulation/start` | POST | Auditeur |
| `/simulation/stop` | POST | Décideur |
| `/simulation/phase_ack` | POST | Décideur |

## Configuration

`config/config.json` :

- `dry_run` : `true` en développement
- `sandbox_cidr`, `production_cidrs`
- `phase1_scenario`, `phase2_scenario`, `phase3_scenario`
- `require_authorization`, `require_snapshot`, `require_rollback_plan`
- `enforce_launch_window`, `enforce_health_checks`
- `global_max_duration_secs` : **900**

Variables : `SIMULATEUR_CONFIG`, `SIMULATEUR_DRY_RUN`.

## Lancer

```bash
cargo run
# écoute sur 0.0.0.0:8005
```

## Exemple lancement (prod-like)

```bash
curl -X POST http://127.0.0.1:8005/simulation/start \
  -H "Content-Type: application/json" \
  -H "X-Agent-Token: CHANGE_ME" \
  -d '{
    "scenario": "brute_force",
    "target": { "type": "vm", "value": "192.168.99.10" },
    "authorized_by": "admin_01",
    "authorization_ref": "AUTH-2026-042",
    "snapshot_id": "snap-abc123",
    "rollback_plan_id": "rb-plan-7",
    "max_duration_seconds": 120
  }'
```

## Structure

- `knowledge/attacks_catalog.json` — base de connaissances attaques + timeouts
- `src/control/` — FSM, orchestrateur, contraintes C1–C15
- `src/comms/` — clients HTTP agents + health C7/C15
- `src/scenarios/` — catalogue + executor (25 scénarios)
- `src/api/` — serveur Axum

## Références

- `Document_APIs.txt` — contrat REST
- `Gandal_phase0.txt` — anomalies ANO-xxx, règle `simulation: true`
