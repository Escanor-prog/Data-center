use tokio::process::Command;
use tracing::{info, warn, error};

/// Exécute un scénario de simulation de manière asynchrone et sécurisée.
///
/// Cette fonction prend le nom du scénario, l'IP cible et la durée maximale.
/// Afin de garantir la sécurité et d'éviter les actions offensives réelles dans l'environnement de test,
/// chaque attaque est simulée via un wrapper exécutant une commande `echo` montrant la commande fictive
/// qui aurait été lancée (nmap, hping3, hydra, arpspoof, etc.).
///
/// La commande `echo` est configurée pour fonctionner sur Windows (`cmd /C echo`) et Unix (`echo`).
pub async fn execute(
    scenario: &str,
    target_ip: &str,
    duration_secs: u64,
) -> Result<(), String> {
    info!(
        scenario = %scenario,
        target = %target_ip,
        duration = duration_secs,
        "Démarrage de la simulation du scénario"
    );

    // Définition de la commande mockée pour chaque type de scénario
    let mock_command = match scenario {
        "ddos_syn" => {
            format!("hping3 -S --flood -V {}", target_ip)
        }
        "scan_ports" => {
            format!("nmap -sS {}", target_ip)
        }
        "brute_force" => {
            format!("hydra -l admin -P /opt/wordlists/rockyou.txt ssh://{}", target_ip)
        }
        "exfiltration" => {
            format!("curl -X POST -H \"Content-Type: application/octet-stream\" -d @db_dump.sql http://{}/exfil", target_ip)
        }
        "mitm" => {
            format!("arpspoof -i eth0 -t {} 10.0.0.1", target_ip)
        }
        "replay_attack" => {
            format!("curl -H \"Authorization: Bearer EXPIRED_TOKEN_ABC123\" http://{}/api/transactions", target_ip)
        }
        "saturation" => {
            format!("dd if=/dev/zero of=/var/tmp/sandbox_saturation.img bs=1M count=100 timeout_secs={}", duration_secs)
        }
        "monitoring_fail" => {
            format!("systemctl stop monitoring-agent.service (target={})", target_ip)
        }
        "panne_reseau" => {
            format!("ip link set dev eth0 down (target={})", target_ip)
        }
        "cert_tls" => {
            format!("openssl s_client -connect {}:443 -cert expired_cert.pem", target_ip)
        }
        "mot_de_passe_faible" => {
            format!("curl -X POST -d \"{{\\\"user\\\":\\\"admin\\\",\\\"pass\\\":\\\"123456\\\"}}\" http://{}/api/auth", target_ip)
        }
        "injection_base" => {
            format!("curl -G \"http://{}/rules\" --data-urlencode \"query=UNION SELECT ALL\"", target_ip)
        }
        _ => {
            let err_msg = format!("Scénario inconnu ou non implémenté : {}", scenario);
            error!("{}", err_msg);
            return Err(err_msg);
        }
    };

    info!(
        command = %mock_command,
        "Préparation de l'exécution de la commande de simulation (mock)"
    );

    // Initialisation portable de la commande (gestion de cmd.exe sous Windows)
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", "echo"]);
        c
    } else {
        Command::new("echo")
    };

    // Configuration de l'argument à afficher
    let display_str = format!(
        "[SIMULATION MOCK] Exécution de la commande : {} (Durée max : {}s)",
        mock_command, duration_secs
    );
    cmd.arg(&display_str);

    // Exécution asynchrone
    match cmd.output().await {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                info!(
                    scenario = %scenario,
                    stdout = %stdout,
                    "Simulation exécutée avec succès"
                );
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let err_msg = format!("Échec de la commande de simulation : {}", stderr);
                error!(error = %err_msg);
                Err(err_msg)
            }
        }
        Err(e) => {
            let err_msg = format!("Erreur système lors du lancement du processus : {}", e);
            error!(error = %err_msg);
            Err(err_msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_known_scenario() {
        let result = execute("scan_ports", "127.0.0.1", 30).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_unknown_scenario() {
        let result = execute("scenario_inexistant", "127.0.0.1", 30).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Scénario inconnu ou non implémenté : scenario_inexistant"
        );
    }
}
