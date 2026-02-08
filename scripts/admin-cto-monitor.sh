#!/bin/bash
# Admin CTO Cluster Monitoring TMUX Session

set -euo pipefail

SESSION="admin-cto-monitor"
KUBECONFIG="$HOME/.kube/admin-cto-config"

if ! command -v tmux &> /dev/null; then
    echo "ERROR: tmux not installed"
    exit 1
fi

if tmux has-session -t "$SESSION" 2>/dev/null; then
    echo "Session exists. Attach: tmux attach -t $SESSION"
    exit 0
fi

echo "Creating TMUX session '$SESSION'..."

tmux new-session -d -s "$SESSION" -n 'admin-cto'

# Create pane layout
tmux split-window -v -t "$SESSION:0" -p 66
tmux split-window -v -t "$SESSION:0.1" -p 50
tmux split-window -h -t "$SESSION:0.0" -p 30
tmux split-window -h -t "$SESSION:0.2" -p 30

# Pane titles
tmux select-pane -t "$SESSION:0.0" -T "Pod Status"
tmux select-pane -t "$SESSION:0.1" -T "Node Resources"
tmux select-pane -t "$SESSION:0.2" -T "Hubble Flows"
tmux select-pane -t "$SESSION:0.3" -T "Cilium Status"
tmux select-pane -t "$SESSION:0.4" -T "Interactive Shell"

# Enable pane borders
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format " #{pane_title} "

# Pane 0: Pod Status
tmux send-keys -t "$SESSION:0.0" "export KUBECONFIG=$KUBECONFIG" C-m
tmux send-keys -t "$SESSION:0.0" "watch -n 3 'kubectl get pods -A | grep -v Running | head -20'" C-m

# Pane 1: Node Resources
tmux send-keys -t "$SESSION:0.1" "export KUBECONFIG=$KUBECONFIG" C-m
tmux send-keys -t "$SESSION:0.1" "watch -n 5 'kubectl top nodes && echo && kubectl top pods -A --sort-by=memory | head -10'" C-m

# Pane 2: Hubble External Traffic
tmux send-keys -t "$SESSION:0.2" "export KUBECONFIG=$KUBECONFIG" C-m
tmux send-keys -t "$SESSION:0.2" "kubectl port-forward -n kube-system svc/hubble-relay 4245:80 >/dev/null 2>&1 &" C-m
tmux send-keys -t "$SESSION:0.2" "sleep 3" C-m
tmux send-keys -t "$SESSION:0.2" "hubble observe --server localhost:4245 --follow --not-from-ip 10.0.0.0/8" C-m

# Pane 3: Cilium Status
tmux send-keys -t "$SESSION:0.3" "export KUBECONFIG=$KUBECONFIG" C-m
tmux send-keys -t "$SESSION:0.3" "watch -n 10 'kubectl -n kube-system exec ds/cilium -- cilium status --brief 2>/dev/null || echo Waiting for Cilium...'" C-m

# Pane 4: Interactive Shell
tmux send-keys -t "$SESSION:0.4" "export KUBECONFIG=$KUBECONFIG" C-m
tmux send-keys -t "$SESSION:0.4" "export TALOSCONFIG=$HOME/.talos/admin-cto-config" C-m
tmux send-keys -t "$SESSION:0.4" "cd /Users/jonathonfritz/agents/main" C-m
tmux send-keys -t "$SESSION:0.4" "kubectl get nodes" C-m

tmux select-pane -t "$SESSION:0.4"

echo "✓ Session created!"
echo "Attach: tmux attach -t $SESSION"
