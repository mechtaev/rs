#!/bin/bash

# Before using this script, create a public key with empty password using ssh-keygen, 
# then add your ~/.ssh/id_rsa.pub to ~/.ssh/authorized_keys for all remote nodes.

USER="sadm"
NODES=(genprogexp-1-i.comp.nus.edu.sg
       genprogexp-2-i.comp.nus.edu.sg
       genprogexp-3-i.comp.nus.edu.sg
       genprogexp-4-i.comp.nus.edu.sg
       genprogexp-5-i.comp.nus.edu.sg
       genprogexp-6-i.comp.nus.edu.sg
       genprogexp-7-i.comp.nus.edu.sg
       genprogexp-8-i.comp.nus.edu.sg
       genprogexp-9-i.comp.nus.edu.sg
       genprogexp-10-i.comp.nus.edu.sg)
PORT=22 # 3022 for my localhost VM

SESSION=seryozha-experiments

SSH="ssh -q -p $PORT"
SCP="scp -q -P $PORT"

SUCCEEDED=".task-succeeded"
FAILED=".task-failed"

CO='\e[0;34m'
NC='\e[0m' # No Color

start_session() {
    node=$1
    $SSH $USER@$node "screen -dmS $SESSION"
}

stop_session() {
    node=$1
    $SSH $USER@$node "screen -S $SESSION -p 0 -X kill"
}

get_screen() {
    node=$1
    tmp_remote=`$SSH $USER@$node mktemp`
    tmp_local=`mktemp`
    $SSH $USER@$node "screen -S $SESSION -p 0 -X hardcopy $tmp_remote"
    $SCP $USER@$node:$tmp_remote $tmp_local
    cat $tmp_local
}

exec_remote() {
    node=$1
    cmd="$2; if [[ \\\$? -eq 0 ]]; then touch ~/$SUCCEEDED; else touch ~/$FAILED; fi"
    $SSH $USER@$node "rm ~/$SUCCEEDED ~/$FAILED 2> /dev/null; screen -S $SESSION -p 0 -X stuff \"$cmd\"; screen -S $SESSION -p 0 -X eval 'stuff \\015'"
}

send_sigint() {
    node=$1
    $SSH $USER@$node "screen -S $SESSION -p 0 -X eval 'stuff \\003'"
}

check_status() {
    node=$1
    printf "Node: %s\n" $node 
    status=$($SSH $USER@$node "[[ -f ~/$SUCCEEDED ]] && echo SUCCESS; [[ -f ~/$FAILED ]] && echo FAIL")
    if [[ -n "$status" ]]; then
        printf "%s\n" "Status: $status"
    else
        printf "%s\n" "Status: IN PROGRESS"
    fi
    printf "${CO}" ""; get_screen $node; printf "${NC}" ""
}


case "$1" in
    up)
        for node in "${NODES[@]}"; do
            start_session $node
        done
        ;;
    down)
        for node in "${NODES[@]}"; do
            stop_session $node
        done
        ;;
    exec)
        for node in "${NODES[@]}"; do
            exec_remote $node "$2"
        done
        ;;
    sigint)
        for node in "${NODES[@]}"; do
            send_sigint $node
        done
        ;;
    status)
        for node in "${NODES[@]}"; do
            check_status $node
        done
        ;;
    *)
        echo "Usage: `basename $0` { up | down | exec CMD | sigint | status }"
        exit 0
        ;;
esac
