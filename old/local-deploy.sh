#!/usr/bin/env bash

projectName=$(git remote get-url --push origin | grep -oE '([^/]*)(.git)?$' | grep -oE '^[^.]*')
currentBranch=$(git branch --list | grep master | tr -d "* ")
git add .
gitStatus=`git status -s`
if [ ! -z "${gitStatus}" ]
then
  echo "set commit message:"
  read arg
  echo "the message is "${arg}
  git commit -m "${arg}"
  git push origin "${currentBranch}"
fi

git push origin ${currentBranch}:test --force
expect -c "
  set timeout 10
  spawn ssh root@qctest1 \"/data/software/hnqc/deploy-test.sh ${projectName}\"
  expect {
    \"yes/no\" { send \"yes\n\";exp_continue }
    \"password\" { send \"testQclisgood2017\r\" }
  }
  interact
  "