#!/usr/bin/env bash

SOURCE="$0"
while [ -h "$SOURCE"  ]; do # resolve $SOURCE until the file is no longer a symlink
    DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"
    SOURCE="$(readlink "$SOURCE")"
    [[ $SOURCE != /*  ]] && SOURCE="$DIR/$SOURCE" # if $SOURCE was a relative symlink, we need to resolve it relative to the path where the symlink file was located
done
DIR="$( cd -P "$( dirname "$SOURCE"  )" && pwd  )"

source "${DIR}/prod.config"

if [ ! -e "$localGitDir" ]
then
  mkdir -p  "$localGitDir"
fi

if [ ! -e "$localBinDir" ]
then
  mkdir -p "$localBinDir"
fi

if [ ! -e "$localTempDir" ]
then
  mkdir -p "$localTempDir"
fi

if [ ! -e "$localLogDir" ]
then
  mkdir -p "$localLogDir"
fi

# need provide full path,root path and target name.warn:no ".jar"
packageProject(){
  packageName="${3}.jar"
  projectPath="${localGitDir}/$2"
  cd "$projectPath"
  echo "packaging project to ${localGitDir}..."
  `${packageCommand}`
  jarLocation=${localGitDir}/${fullpath}/${packageTarget}/${packageName}
  if [ ! -e "${jarLocation}" ]
  then
    echo "package error, deploy failed."
    exit 1
  fi
  echo "package success"
  # copy original jar
  if [ -e "${localBinDir}/${packageName}" ]
  then
    mv "${localBinDir}/${packageName}" "${localTempDir}"
  fi
  # copy new jar to bin dir
  mv "${jarLocation}" "${localBinDir}"
}

# need provide projectName
shutdown(){
  PID=$(ps -ef | grep ${1}.jar | grep -v grep | awk '{ print $2 }')
  if [ -z "${PID}" ]
  then
    echo "original proccess is already stopped"
  else
    echo kill ${PID}
    kill -9 ${PID}
  fi
}

# need provide projectName
startup(){
  echo "starting ${1}..."
  cd ${localBinDir}
  if [ ! -e "${localLogDir}/$1.log" ]
  then
    touch "${localLogDir}/$1.log"
  fi

  $(nohup "${localJavaBin}" -Xms64m -Xmx128m -jar "${localBinDir}/$1.jar"  > "${localLogDir}/$1.log" &)
  sleep 1s
  today=`date +%Y-%m-%d`
  hour=`date +%H`
  while [ -f "${localLogDir}/$1.log" ]
  do
    result=$(grep "${today} ${hour}" "${localLogDir}/$1.log" | grep "Started")
    if [ ! -z "${result}" ]
    then
      echo "start $1 successed"
      break
    else
      result=$(grep "${today} ${hour}" "${localLogDir}/$1.log" | grep "ERROR")
      if [ ! -z "$result" ]
      then
        shutdown ${1}
        echo "start ${1} error,the log file will move to ${localLogDir}/${1}-error.log,execution rollback"
        mv "${localLogDir}/$1.log" "${localLogDir}/$1-error.log"
        rollback ${1}
        echo "please see ${localLogDir}/$1-error.log to fix problems"
        break
      fi
      sleep 1s
    fi
  done
}
# need provide projectName
rollback(){
  if ! test -e "${localTempDir}/$1.jar"
  then
    echo "warn : no project to rollback"
    return
  fi
  mv "${localTempDir}/$1.jar" "${localBinDir}"
  nohup "${localJavaBin}" -Xms64m -Xmx128m -jar "${localBinDir}/$1.jar" > "${localLogDir}/$1.log" &
  echo "$1 rollbacked"
}

# need provide git path and local project path.eg:pullProject remote project
pullProject(){
  remoteGitPath=$1
  localProjectPath=$2
# $localGitDir"/"$projectName

  echo "pulling ${remoteGitPath}..."
  if ! test -e ${localProjectPath}
  then
    git clone "${remoteGitPath}" "${localProjectPath}"
    cd "${localProjectPath}"
    git checkout -t "${remoteName}/${branchName}"
  else
    cd "${localProjectPath}"
    git fetch --all > /dev/null 2>&1
    git reset --hard "${remoteName}/${branchName}" > /dev/null 2>&1
  fi
  echo "finish pulling ${remoteGitPath}"
}

# need provide local project path,root path and projectName.eg:validateProject path and project Name
validateProject(){
  localProjectPath="${localGitDir}/${1}"
  rootpath="${localGitDir}/${2}"
  projectName=${3}
  echo "validate project"
  if [ ! -e "${localProjectPath}/src/main/resources/application-${packageEnv}.properties" ];then
    echo "error:source file ${localProjectPath}/src/main/resources/application-${packageEnv}.properties does not exsits"
    exit 1
  fi
  if ! test -e "${localProjectPath}/pom.xml"
  then
    echo "${localProjectPath}/pom.xml does not exsits"
    exit 1
  fi
  fixPackageName ${localProjectPath}"/pom.xml" ${projectName}
  pomarr=($(find ${rootpath} -name pom.xml))
  for pomp in ${pomarr[@]}
  do
    fixDependeces ${pomp}
  done
  fixProperties "${localProjectPath}/src/main/resources/application.properties"
}

fixProperties(){
  propertiesPath=$1
  if ! test -e "${propertiesPath}"
  then
    touch "$propertiesPath"
  fi
  # fix default sring.profiles.active
  echo "set spring.profiles.active -> "${packageEnv}
  sed -i "/^\s*spring.profiles.active/d" "${propertiesPath}"
  echo -e "\nspring.profiles.active=${packageEnv}" >> "${propertiesPath}"
#  sed -i "s/^\s*spring.profiles.active=.*/spring.profiles.active=${packageEnv}/g" ${propertiesPath}
}

fixPackageName(){
  pomPath=$1
  projectName=$2

  regex="<finalName\s*>\s*[^<]*"
  replacement="<finalName>${projectName}"

  if [ ! -z `sed -n "s/${regex}/${replacement}/p" "${pomPath}"` ]
  then
    echo "replace finalName ${projectName} to pom.xml [project/build/finalName]"
    sed -i "s/${regex}/${replacement}/g" ${pomPath}
  else
    echo "append finalName ${projectName} to pom.xml [project/build/finalName]"
    sed -i "s/<build\s*>/<build><finalName>${projectName}<\/finalName>/g" ${pomPath}
  fi
}
# need provide pomPath
fixDependeces(){
  pomPath=$1
  # fix dependences
  OLD_IFS="$IFS"
  text=`cat ${pomPath} | tr -d '\n'`
  for dep in "${dependencies[@]}"
  do
    IFS=":"
    arr=($dep)
    #恢复原来的分隔符
    IFS="$OLD_IFS"
    groupId=${arr[0]//"."/"\."}
    artifactId=${arr[1]//"."/"\."}
    version=""
    if [ ${#arr[*]} == 3 ]
    then
      version=${arr[2]//"."/"\."}
    fi
    rm ${mvnRepository}"/"${arr[0]//"."/"/"}"/"${arr[1]} -rf
    pattern="(<groupId\s*>\s*${groupId}\s*<\/groupId\s*>\s*|\s*<artifactId\s*>\s*${artifactId}\s*<\/artifactId\s*>\s*|\s*<version\s*>[^<]+<\/version\s*>){3}"
    replacement="<groupId>${groupId}<\/groupId><artifactId>${artifactId}<\/artifactId><version>${version}<\/version>"
    info=`echo $text | sed -r -n "s/${pattern}/${replacement}/p"`
    if [[ ! -z ${info} ]]
    then
      echo "fix dependency: "${groupId//"\."/"."}:${artifactId//"\."/"."}" -> "${version//"\."/"."}
      text=${info}
    fi
  done
  echo $text > ${pomPath}
}

# deploy,need provide remote url, full path and project name
deploy(){
  fullpath=$2
  # 使用 / 分割项目
  OLD_IFS="$IFS" 
  IFS="/" 
  namearr=($2) 
  IFS="$OLD_IFS"
  proName=${namearr[${#namearr[*]}-1]}
  echo ">>>deploy project $proName started"
  # pull project
  pullProject $1 $localGitDir"/"${namearr[0]}
  # validate
  validateProject $fullpath ${namearr[0]} $proName
  packageProject $fullpath ${namearr[0]} $proName
  shutdown $proName
  startup $proName
  echo "deploy project $fullpath finished"
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
  fullpath=$name
  # 使用 / 分割项目
  OLD_IFS="$IFS" 
  IFS="/" 
  varr=($name) 
  IFS="$OLD_IFS"
  name=${varr[0]}
  deploy "git@gitlab.scustartup.com:hnqc/${name}.git" ${fullpath} 
done
