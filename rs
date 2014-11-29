#!/bin/bash
#
# rs
# Copyright (C) 2014 Sergey Mechtaev
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program. If not, see <http://www.gnu.org/licenses/>.


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

IFS=: read -a NODES <<< $RS_CLUSTER

SSH="ssh -q -p $RS_PORT"
SCP="scp -q -P $RS_PORT"

CO='\e[0;34m'
NC='\e[0m' # No Color

start_session() {
    session=$1
    node=$2
    status=$($SSH $RS_USER@$node "screen -dmS $session" 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    else
        echo "Session $session is started at $node"
    fi
}

stop_session() {
    session=$1
    node=$2
    status=$($SSH $RS_USER@$node "screen -S $session -p 0 -X kill" 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    else
        echo "Session $session is stoped at $node"
    fi
}

get_screen() {
    session=$1
    node=$2
    tmp_remote=`$SSH $RS_USER@$node mktemp`
    tmp_local=`mktemp`
    status=$($SSH $RS_USER@$node "screen -S $session -p 0 -X hardcopy $tmp_remote" 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    fi
    status=$($SCP $RS_USER@$node:$tmp_remote $tmp_local 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    fi
    cat $tmp_local | grep -v '^$'
}

send_sigint() {
    session=$1
    node=$2
    status=$($SSH $RS_USER@$node "screen -S $session -p 0 -X eval 'stuff \\003'" 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    fi
}

send_command() {
    session=$1
    node=$2
    status=$($SSH $RS_USER@$node "screen -S $session -p 0 -X stuff \"$3\"; screen -S $session -p 0 -X eval 'stuff \\015'" 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    fi
}

exec_remote() {
    session=$1
    node=$2
    succeeded=".${session}-task-succeeded"
    failed=".${session}-task-failed"
    cmd="clear; $3; if [[ \\\$? -eq 0 ]]; then touch ~/$succeeded; else touch ~/$failed; fi"
    status=$($SSH $RS_USER@$node "rm ~/$succeeded ~/$failed 2> /dev/null" 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    fi
    send_command $session $node "$cmd"
}

send_files() {
    node=$1
    from=$2
    to=$3
    status=$($SCP -r $from $RS_USER@$node:$to 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    fi
}

receive_files() {
    node=$1
    from=$2
    to=$3
    status=$($SCP -r $RS_USER@$node:$from $to 2>&1)
    if [[ ! -z $status ]]; then
        echo "Error at ${node}: $status"
    fi
}

check_status() {
    session=$1
    node=$2
    succeeded=".${session}-task-succeeded"
    failed=".${session}-task-failed"
    status=$($SSH $RS_USER@$node "[[ -f ~/$succeeded ]] && echo SUCCESS; [[ -f ~/$failed ]] && echo FAIL")
    if [[ -n "$status" ]]; then
        printf "%s at %s (session %s)\n" "$status" "$node" "$session"
    else
        printf "%s at %s (session %s)\n" "IN PROGRESS" "$node" "$session"
    fi
    printf "${CO}" ""; get_screen $session $node; printf "${NC}" ""
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
                command="$3"
                for node in $NODES; do
                    exec_remote $session $node "$command"
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
                if [ -d "$destination" ]; then
                    echo "ERROR: destination should be a file"
                    exit -1
                fi
                for node in $NODES; do
                    receive_files $node $source "${destination}-$node"
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
                command="$3"
                for node in $NODES; do
                    send_command $session $node "$command"
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
        echo "Bash script for running commands on a cluster using screen"
        echo "Usage: `basename $0` CMD"
        echo ""
        echo "CMD"
        echo "    start SESSION                 start remote screen sessions"
        echo "    stop SESSION                  terminate remote screen sessions"
        echo "    exec SESSION CMD              execute CMD in each remote screen session and save its exit status"
        echo "    upload SOURCE DESTINATION     copy file SOURCE to each node to location DESTINATION"
        echo "    download SOURCE DESTINATION   copy file SOURCE to each node to location DESTINATION"
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
