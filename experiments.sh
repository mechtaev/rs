#!/bin/bash

# Before using this script, create a public key with empty password using ssh-keygen, 
# then add your ~/.ssh/id_rsa.pub to ~/.ssh/authorized_keys for all remote nodes.

USER="semfix"
NODE=127.0.0.1
PORT=3022

SSH="ssh -q -p $PORT"
SCP="scp -q -P $PORT"

SUCCEEDED=".task-succeeded"
FAILED=".task-failed"

CO='\e[0;34m'
NC='\e[0m' # No Color

start_session() {
    $SSH $USER@$NODE "screen -d -m"
}

stop_session() {
    $SSH $USER@$NODE "screen -p 0 -X kill"
}

get_screen() {
    tmp_remote=`$SSH $USER@$NODE mktemp`
    tmp_local=`mktemp`
    $SSH $USER@$NODE "screen -p 0 -X hardcopy $tmp_remote"
    $SCP $USER@$NODE:$tmp_remote $tmp_local
    cat $tmp_local
}

exec_remote() {
    cmd="$1; if [[ \\\$? -eq 0 ]]; then touch ~/$SUCCEEDED; else touch ~/$FAILED; fi"
    $SSH $USER@$NODE "rm ~/$SUCCEEDED ~/$FAILED 2> /dev/null; screen -p 0 -X stuff \"$cmd\"; screen -p 0 -X eval 'stuff \\015'"
}

check_status() {
    printf "Node: %s\n" $NODE 
    status=$($SSH $USER@$NODE "[[ -f ~/$SUCCEEDED ]] && echo SUCCESS; [[ -f ~/$FAILED ]] && echo FAIL")
    if [[ -n "$status" ]]; then
        printf "%s\n" "Status: $status"
    else
        printf "%s\n" "Status: IN PROGRESS"
    fi
    printf "${CO}" ""; get_screen; printf "${NC}" ""
}


case "$1" in
    start)
        start_session
        ;;
    stop)
        stop_session
        ;;
    exec)
        exec_remote "$2"
        ;;
    submit)
        submit_task "$2"
        ;;
    status)
        check_status
        ;;
    *)
        echo "Usage: `basename $0` { start | stop | exec CMD | submit TASK | status }"
        exit 0
        ;;
esac
