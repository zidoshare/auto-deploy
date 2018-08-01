# 工作流程

1. 开发者根据环境提交到git分支[正式环境：online,测试环境：test]，如有特殊处理的依赖，可以修改[test.config/prod.config]配置文件中的dependencies项。
2. 部署时开发者登录服务器，执行正式环境/测试环境部署shell，每个项目需要单独提供参数 `projectName`，可提供多个项目名[项目名为git线上项目名]，多个项目名空格隔开。不填写项目名不执行shell。
3. shell拉取线上对应分支
4. shell根据环境校验并切换配置文件
5. shell执行maven打包（默认跳过java测试，默认打包名为前面提供的项目名而不是pom定义的打包项目名），打包时默认不输出打包信息，报错时输出报错信息并停止部署
6. shell将打包文件提交到服务器对应目录[正式/测试不同服务器]，并备份原jar包
7. shell自动部署项目，监控项目启动，当出现报错时，自动回滚项目(仅针对部署时的导致部署失败的错误)

## 关于多模块项目

`projectName` 需要包含根目录及启动目录，支持多级多模块，例如：`hnqc-shouyi/shouyi-server-customer`、`hnqc-shouyi/customer/server`

## 本地快速执行

注意:__不支持windows平台__

__多模块项目需要自行修改projectName__,
将
```shell
projectName=$(git remote get-url --push origin | grep -oE '([^/]*)(.git)?$' | grep -oE '^[^.]*')
```
修改为
```shell
projectName=$(git remote get-url --push origin | grep -oE '([^/]*)(.git)?$' | grep -oE '^[^.]*')/这里写启动模块名称
```

下载[local-deploy.sh](http://gitlab.scustartup.com/zido/auto-deploy/blob/master/local-deploy.sh)脚本到__项目内__的任意路径即可。

### 特征

* 自动寻找项目名称，不用填写（必须保证在项目路径下），原理是获取git的origin分支url，截取url(如果url包含.git后缀，也会忽略掉这个后缀)

* 自动找当前git分支，但是最好不要在[test/online]发布分支开发，因为发布分支的代码更改会被强制覆盖。

* 如果当前分支代码有更改但未提交，自动提示需要填写commit信息，并且必须填写，否则停止部署

* 自动输入密码连接服务器执行发布流程（linux/mac平台需要安装expect命令行工具）

## 打包条件

* 项目目录需符合以下目录结构(多模块项目保持启动模块拥有此结构即可)

```code
|-root
|  |--src
|  |   |--main
|  |   |   |--resources
|  |   |   |     |--application.properties [总控配置文件，可选]
|  |   |   |     |--application-prod.properties [正式环境配置文件,必选]
|  |   |---|-----|--application-test.properties [测试环境配置文件,必选]
|  |   |   |     |
|--|--pom.xml
```

* `application.properties`文件中`spring.profiles.active`会被忽略，由shell自动填充替换
* pom.xml文件中的`finalName`会被忽略，最终构建名会由shell自动填充替换
* pom.xml文件中的依赖，如果被配置文件[config文件中的dependencies项]中的规定的依赖所匹配，将自动根据环境切换，如果未匹配则无改变，配置文件依赖组需严格按照格式编写,数组除分隔元素使用空格外，其他地方不能包含任何空格（元素中不算）。因多行正则的限制，不采用maven中的依赖写法，而是采用gradle依赖写法`<groupId>:<artifactId>:<version>`，此处如果未写version仍然会被匹配替换（考虑到依赖管理可能包含版本）,例子:

```shell
dependencies=("com.hnqc:hnqc_common:0.0.1" "com.hnqc:hnqc_search:0.0.1")

```

* 开发者需在服务器上执行打包命令
* 如想要简单的本地执行可以复制以下命令（linux/mac）:`ssh root@qctest1 "/data/software/hnqc/deploy-test.sh <project...>"`。project换成项目名即可。