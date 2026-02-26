#[derive(Debug, Clone)]
pub enum GuardDecision {
    Allow,
    Confirm(String),
    Block(String),
}

pub fn evaluate_command(cmd: &str) -> GuardDecision {
    let c = cmd.trim();
    if c.is_empty() {
        return GuardDecision::Allow;
    }

    let hard_block = [":(){:|:&};:", "mkfs", "shutdown", "reboot"];
    if hard_block.iter().any(|token| c.contains(token)) {
        return GuardDecision::Block("command matches blocked host-level pattern".to_string());
    }

    let confirm = ["rm -rf", "kubectl delete ns", "kubectl delete namespace"];
    if confirm.iter().any(|token| c.contains(token)) {
        return GuardDecision::Confirm("high-risk command detected".to_string());
    }

    GuardDecision::Allow
}
