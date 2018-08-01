#!/usr/bin/env bash

# need provide projectName
shutdown(){
  PID=$(ps -ef | grep $1.jar | grep -v grep | awk '{ print $2 }')
  if [ -z "$PID" ]
  then
    echo "original proccess is already stopped"
  else
    echo kill $PID
    kill -9 $PID
  fi
}
# get opts
if [ $# -lt 1 ]
then
  echo "need args"
  exit 1
fi
for name in "$@"
do
  if [[ $name == -* ]]
  then
    echo "invalid arg : $name,and skip it "
    continue
  fi
  shutdown $name
done