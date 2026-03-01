/// Returns a short educational analogy for a CLI command.
///
/// Parses the command string and matches against known kubectl subcommands,
/// resource types, and common Kubernetes tooling.
pub fn command_blurb(cmd: &str) -> &'static str {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return "A mystery command — try running it to find out";
    }

    let tool = parts[0];

    // Non-kubectl tools — match on the binary name
    match tool {
        "kind" => return "A snow globe cluster you shake up on your laptop",
        "brew" => return "The vending machine for every dev tool you need",
        "helm" => return "Like ordering from IKEA — chart, flat-pack, assemble",
        "kustomize" => return "Edit the master template without touching the original",
        "kubeadm" => return "The hard-hat crew that pours the cluster foundation",
        "etcdctl" => return "Rifling through the cluster's single source of truth",
        "docker" => return "Shrink-wrapping your app into a shipping container",
        "curl" => return "Knocking on a door to see if anyone answers",
        _ => {}
    }

    if tool != "kubectl" || parts.len() < 2 {
        return tool_fallback(tool);
    }

    let subcommand = parts[1];

    // If there's a resource type (e.g. "kubectl get pods"), check it
    if parts.len() >= 3
        && let Some(blurb) = resource_blurb(parts[2]) {
            return blurb;
        }

    // Match on kubectl subcommand
    subcommand_blurb(subcommand)
}

fn subcommand_blurb(sub: &str) -> &'static str {
    match sub {
        "get" => "The guest list — who's here and who's missing",
        "describe" => "The full incident report, not just the headline",
        "logs" => "Reading a ship's logbook after the voyage",
        "exec" => "Climbing inside the machine while it's running",
        "delete" => "Filing the eviction notice and walking away",
        "run" => "Spinning up a food truck on an empty corner",
        "apply" => "Handing the city blueprints to the building crew",
        "create" => "Filing the paperwork to open a new business",
        "edit" => "Rewriting a live contract while everyone watches",
        "patch" => "Slapping a Post-it note over the old instruction",
        "rollout" => "The stage director managing a scene in motion",
        "scale" => "Calling in extra staff during the dinner rush",
        "label" => "Slapping a price tag sticker on shelf inventory",
        "annotate" => "Writing sticky notes on the back of a photograph",
        "top" => "The electricity meter spinning on the wall",
        "cordon" => "Roping off a lane — no new traffic allowed",
        "uncordon" => "Removing the roadblock — lane open again",
        "drain" => "Evacuating tenants before the building demo",
        "taint" => "Hanging a Biohazard sign — most keep out",
        "port-forward" => "A secret tunnel dug under the castle wall",
        "cp" => "Dropping files through a slot in the door",
        "auth" => "Checking if your badge opens the restricted wing",
        "explain" => "Pulling the IKEA manual mid-build",
        "config" => "Swapping the master key ring at the front desk",
        "cluster-info" => "The hospital lobby board — all systems status",
        "api-resources" => "The menu of every dish this kitchen can make",
        "version" => "Checking the firmware sticker on the router",
        _ => "A kubectl spell — check the docs for details",
    }
}

fn resource_blurb(resource: &str) -> Option<&'static str> {
    // Normalize: strip trailing 's' for singular forms, handle common aliases
    let normalized = resource
        .strip_suffix("es")
        .or_else(|| resource.strip_suffix('s'))
        .unwrap_or(resource);

    match normalized {
        "pod" => Some("Individual workers clocked in on the factory floor"),
        "node" => Some("Physical warehouses where all the workers live"),
        "namespace" => Some("Floors in the building — same address, separate spaces"),
        "service" | "svc" => Some("The receptionist routing visitors to the right desk"),
        "deployment" | "deploy" => Some("The HR contract: how many staff, which version"),
        "configmap" | "cm" => Some("The laminated reference card taped above the register"),
        "secret" => Some("The manager's safe — same room, much heavier lock"),
        "ingress" | "ing" => Some("The lobby directory pointing to the right floor"),
        "networkpolic" | "networkpolicy" | "netpol" => {
            Some("The velvet rope deciding who may talk to whom")
        }
        "pv" | "persistentvolume" => Some("A rented storage unit that outlives any tenant"),
        "pvc" | "persistentvolumeclaim" => Some("The signed lease claim on that storage unit"),
        "serviceaccount" | "sa" => Some("A staff badge granting specific room access only"),
        "role" => Some("The rulebook for what one floor's staff may do"),
        "rolebinding" => Some("Handing that rulebook to a specific employee"),
        "clusterrole" => Some("A master key policy for the whole building"),
        "clusterrolebinding" => Some("Issuing that master key to a specific person"),
        "event" => Some("Security camera footage from the past hour"),
        "storageclass" | "sc" => Some("The catalogue of storage tiers: cheap to SSD"),
        "daemonset" | "ds" => Some("One guard posted at every gate in the building"),
        "statefulset" | "sts" => Some("Numbered lockers — each worker keeps their own"),
        "replicaset" | "rs" => Some("The staffing agency ensuring the right headcount"),
        "job" => Some("A temp worker: do the task, then clock out"),
        "cronjob" | "cj" => Some("The alarm clock that hires a temp on schedule"),
        "endpoint" | "ep" => Some("The actual phone numbers behind the switchboard"),
        _ => None,
    }
}

fn tool_fallback(tool: &str) -> &'static str {
    match tool {
        "kubectl" => "Your remote control for the entire cluster",
        "kind" => "A snow globe cluster you shake up on your laptop",
        _ => "A tool in your Kubernetes toolkit",
    }
}
