#!/bin/bash

# Before using this script
# (1) define EXP_ID, EXP_USER, EXP_CLUSTER, EXP_PORT variables
# (2) create a public key with empty password using ssh-keygen, 
# then add your ~/.ssh/id_rsa.pub to ~/.ssh/authorized_keys for all remote nodes.

if [[ -z "$EXP_ID" ]]; then
    echo "Specify experiments identifier EXP_ID"
    exit -1
fi

if [[ -z "$EXP_USER" ]]; then
    echo "Specify cluster user EXP_USER"
    exit -1
fi

if [[ -z "$EXP_CLUSTER" ]]; then
    echo "Specify cluster addresses using colon-separated list EXP_CLUSTER"
    exit -1
fi

if [[ -z "$EXP_PORT" ]]; then
    echo "Specify cluster ssh port EXP_PORT"
    exit -1
fi

IFS=: read -a NODES <<<$EXP_CLUSTER

SESSION="${EXP_ID}-experiments"

SSH="ssh -q -p $EXP_PORT"
SCP="scp -q -P $EXP_PORT"

SUCCEEDED=".${EXP_ID}-task-succeeded"
FAILED=".${EXP_ID}-task-failed"

CO='\e[0;34m'
NC='\e[0m' # No Color

start_session() {
    node=$1
    $SSH $EXP_USER@$node "screen -dmS $SESSION"
}

stop_session() {
    node=$1
    $SSH $EXP_USER@$node "screen -S $SESSION -p 0 -X kill"
}

get_screen() {
    node=$1
    tmp_remote=`$SSH $EXP_USER@$node mktemp`
    tmp_local=`mktemp`
    $SSH $EXP_USER@$node "screen -S $SESSION -p 0 -X hardcopy $tmp_remote"
    $SCP $EXP_USER@$node:$tmp_remote $tmp_local
    cat $tmp_local
}

send_sigint() {
    node=$1
    $SSH $EXP_USER@$node "screen -S $SESSION -p 0 -X eval 'stuff \\003'"
}

send_command() {
    node=$1
    $SSH $EXP_USER@$node "screen -S $SESSION -p 0 -X stuff \"$2\"; screen -S $SESSION -p 0 -X eval 'stuff \\015'"
}

exec_remote() {
    node=$1
    cmd="$2; if [[ \\\$? -eq 0 ]]; then touch ~/$SUCCEEDED; else touch ~/$FAILED; fi"
    $SSH $EXP_USER@$node "rm ~/$SUCCEEDED ~/$FAILED 2> /dev/null"
    send_command $node "$cmd"
}

send_files() {
    node=$1
    from=$2
    to=$3
    $SCP -r $from $EXP_USER@$node:$to
}


check_status() {
    node=$1
    printf "Node: %s\n" $node 
    status=$($SSH $EXP_USER@$node "[[ -f ~/$SUCCEEDED ]] && echo SUCCESS; [[ -f ~/$FAILED ]] && echo FAIL")
    if [[ -n "$status" ]]; then
        printf "%s\n" "Status: $status"
    else
        printf "%s\n" "Status: IN PROGRESS"
    fi
    printf "${CO}" ""; get_screen $node; printf "${NC}" ""
}


case "$1" in
    up)
        for node in $NODES; do
            start_session $node
        done
        ;;
    down)
        for node in $NODES; do
            stop_session $node
        done
        ;;
    exec)
        for node in $NODES; do
            exec_remote $node "$2"
        done
        ;;
    es)
        for node in $NODES; do
            exec_remote $node "$2"
        done
        for node in $NODES; do
            check_status $node
        done
        ;;
    copy)
        for node in $NODES; do
            send_files $node "$2" "$3"
        done
        ;;
    sigint)
        for node in $NODES; do
            send_sigint $node
        done
        ;;
    send)
        for node in $NODES; do
            send_command $node $2
        done
        ;;
    status)
        for node in $NODES; do
            check_status $node
        done
        ;;
    help)
        echo "Usage: `basename $0` { up | down | exec CMD | es CMD | copy PATH PATH | send CMD | sigint | status | help}"
        echo "Sending commands to screen sessions on remote hosts."
        echo ""
        echo "up         start remote screen sessions"
        echo "down       terminate remote screen sessions"
        echo "exec CMD   execute CMD in each remote screen session and save its exit status"
        echo "copy S D   copy file S to each node to the location D"
        echo "send CMD   send CMD to each remote screen session followed by ENTER"
        echo "sigint     send SIGINT to each remote screen session"
        echo "status     check status of the last command at each node and print screen fragment"
        echo "es CMD     exec + status"
        echo "help       show this message"
        ;;
    *)
        echo "Usage: `basename $0` { up | down | exec CMD | copy PATH PATH | send CMD | sigint | status | help}"
        exit 0
        ;;
esac
