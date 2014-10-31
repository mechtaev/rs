#!/bin/bash

# Bash script for running commands on a cluster using screen
#
# Before using this script
#   (1) define RS_USER, RS_CLUSTER, RS_PORT variables
#   (2) create a public key with empty password using ssh-keygen,
#       then add your ~/.ssh/id_rsa.pub to ~/.ssh/authorized_keys for all remote nodes.

if [[ -z "$RS_USER" ]]; then
    echo "Specify cluster user RS_USER"
    exit -1
fi

if [[ -z "$RS_CLUSTER" ]]; then
    echo "Specify cluster addresses using colon-separated list RS_CLUSTER"
    exit -1
fi

if [[ -z "$RS_PORT" ]]; then
    echo "Specify cluster ssh port RS_PORT"
    exit -1
fi

IFS=: read -a NODES <<<$RS_CLUSTER

SSH="ssh -q -p $RS_PORT"
SCP="scp -q -P $RS_PORT"

CO='\e[0;34m'
NC='\e[0m' # No Color

start_session() {
    session=$1
    node=$2
    $SSH $RS_USER@$node "screen -dmS $session"
    echo "session $session is started"
}

stop_session() {
    session=$1
    node=$2
    $SSH $RS_USER@$node "screen -S $session -p 0 -X kill"
    echo "session $session is stoped"
}

get_screen() {
    session=$1
    node=$2
    tmp_remote=`$SSH $RS_USER@$node mktemp`
    tmp_local=`mktemp`
    $SSH $RS_USER@$node "screen -S $session -p 0 -X hardcopy $tmp_remote"
    $SCP $RS_USER@$node:$tmp_remote $tmp_local
    cat $tmp_local
}

send_sigint() {
    session=$1
    node=$2
    $SSH $RS_USER@$node "screen -S $session -p 0 -X eval 'stuff \\003'"
}

send_command() {
    session=$1
    node=$2
    $SSH $RS_USER@$node "screen -S $session -p 0 -X stuff \"$3\"; screen -S $session -p 0 -X eval 'stuff \\015'"
}

exec_remote() {
    session=$1
    node=$2
    succeeded=".${session}-task-succeeded"
    failed=".${session}-task-failed"
    cmd="$3; if [[ \\\$? -eq 0 ]]; then touch ~/$SUCCEEDED; else touch ~/$FAILED; fi"
    $SSH $RS_USER@$node "rm ~/$SUCCEEDED ~/$FAILED 2> /dev/null"
    send_command $session $node "$cmd"
}

send_files() {
    node=$1
    from=$2
    to=$3
    $SCP -r $from $RS_USER@$node:$to
}

receive_files() {
    node=$1
    from=$2
    to=$3
    $SCP -r $RS_USER@$node:$from $to
}


check_status() {
    session=$1
    node=$2
    succeeded=".${session}-task-succeeded"
    failed=".${session}-task-failed"
    printf "Node: %s\n" $node
    printf "Session: %s\n" $session
    status=$($SSH $RS_USER@$node "[[ -f ~/$succeeded ]] && echo SUCCESS; [[ -f ~/$failed ]] && echo FAIL")
    if [[ -n "$status" ]]; then
        printf "%s\n" "Status: $status"
    else
        printf "%s\n" "Status: IN PROGRESS"
    fi
    printf "${CO}" ""; get_screen $node; printf "${NC}" ""
}


case "$1" in
    start)
        if [[ -z "$2" ]]; then
            echo "Specify session name"
        else
            session=$2
            for node in $NODES; do
                start_session $session $node
            done
        fi
        ;;
    stop)
        if [[ -z "$2" ]]; then
            echo "Specify session name"
        else
            session=$2
            for node in $NODES; do
                stop_session $session $node
            done
        fi
        ;;
    exec)
        if [[ -z "$2" ]]; then
            echo "Specify session name and command"
        else
            session=$2
            if [[ -z "$3" ]]; then
                echo "Specify command"
            else
                cmd="$3"
                for node in $NODES; do
                    exec_remote $session $node "$cmd"
                done
            fi
        fi
        ;;
    upload)
        if [[ -z "$2" ]]; then
            echo "Specify source and destination"
        else
            source=$2
            if [[ -z "$3" ]]; then
                echo "Specify destination"
            else
                destination="$3"
                for node in $NODES; do
                    send_files $node $source $destination
                done
            fi
        fi
        ;;
    download)
        if [[ -z "$2" ]]; then
            echo "Specify source and destination"
        else
            source=$2
            if [[ -z "$3" ]]; then
                echo "Specify destination"
            else
                destination="$3"
                for node in $NODES; do
                    receive_files $node $source $destination
                done
            fi
        fi
        ;;
    sigint)
        if [[ -z "$2" ]]; then
            echo "Specify session name"
        else
            session=$2
            for node in $NODES; do
                send_sigint $session $node
            done
        fi
        ;;
    send)
        if [[ -z "$2" ]]; then
            echo "Specify session name and command"
        else
            session=$2
            if [[ -z "$3" ]]; then
                echo "Specify command"
            else
                cmd="$3"
                for node in $NODES; do
                    send_command $node "$cmd"
                done
            fi
        fi
        ;;
    status)
        if [[ -z "$2" ]]; then
            echo "Specify session name"
        else
            session=$2
            for node in $NODES; do
                check_status $session $node
            done
        fi
        ;;
    help)
        echo "Usage: `basename $0` CMD"
        echo ""
        echo "CMD"
        echo "    start SESSION                 start remote screen sessions"
        echo "    stop SESSION                  terminate remote screen sessions"
        echo "    exec SESSION CMD              execute CMD in each remote screen session and save its exit status"
        echo "    upload SOURCE DENTINATION     copy file S to each node to the location D"
        echo "    download SOURCE DESTINATION   copy file S to each node to the location D"
        echo "    send SESSION CMD              send CMD to each remote screen session followed by ENTER"
        echo "    sigint SESSION                send SIGINT to each remote screen session"
        echo "    status SESSION                check status of last command at each node and print screen fragment"
        echo "    help                          show this message"
        ;;
    *)
        echo "Usage: `basename $0` { start | stop | exec | upload | download | send | sigint | status | help }"
        exit 0
        ;;
esac
