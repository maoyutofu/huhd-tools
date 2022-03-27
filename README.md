# huhd-tools

一个简单的 Linux Server 文件上传/下载工具。

## 安装
从 release 中下载二进制压缩包
``` shell
tar -xvf huhd-tools-x86_64-unknown-linux-gnu-0.1.0.tar.gz
cd huhd-tools
sudo ./install.sh
```

## 使用

**上传文件**
``` shell
hu
```
在服务器当前目录执行 `hu` 命令就会生成一个临时的 URL，在本地浏览器打开这个 URL 就可以上传文件到服务器当前目录了。更多使用方法 `hu --help`。


**下载文件**
``` shell
hd --file ./test.java
```
在服务器执行 `hd` 命令，通过 `--file` 参数来指定要下载的文件就会生成一个临时的 URL，在本地浏览器打开这个 URL 就可以将文件下载到本地电脑中了。更多使用方法 `hd --help`。

## 环境变量
``` shell
export HUHD_HOST = 192.168.3.11
```
建议在使用前配置该环境变量，该变量的值最好是服务器对外的IP地址/域名。否则默认监听 `0.0.0.0:5567`，在使用时就需要手工将 URL 中的 `0.0.0.0` 替换成你实际的IP/域名使用。