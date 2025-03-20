# vnpyrs-chart

A chart window for both VnpyRS and vnpy

配套VnpyRS或vnpy项目的K线图表窗口

## 介绍

vnpyrs-chart项目的目标是充分利用Rust的高速和GPU的并行化加速K线图表的呈现。
相比于vnpy自带的chart，vnpyrs-chart的速度提升了几个数量级，例如在拥有显卡的电脑上，当K线数量达到50万根的时候，vnpy原本的chart已经超过负荷无法使用，而vnpyrs-chart不会有任何问题，依然保持高帧率无卡顿。
另外，VnpyRS自带一种使用方式，可以直接把vnpyrs-chart对接进vnpy，为vnpy提供极速K线图表的服务，具体方法见VnpyRS的README.md。

## 源码安装

**Windows**

源码安装大体上和Linux一样。也可以从github上下载编译好的可执行文件。

**Linux**

以下以Ubuntu为例

1.拉取源代码
```
git clone https://github.com/vnpyrs/vnpyrs.git
```
若网络不畅，可以从gitee拉取
```
git clone https://gitee.com/vnpyrs/vnpyrs.git
```

2.安装rust

Rust的官网网站是https://www.rust-lang.org/ ，但因为官方的下载速度越来越慢，建议用国内字节的代理，代理的官网是https://rsproxy.cn/
```
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh
```
要求选择时直接回车就行

安装完成后退出shell，重新登陆，键入“cargo”以验证是否安装成功

接下来设置crates.io的镜像，以加快下载Rust依赖包的速度

创建文件~/.cargo/config，内容如下：

```
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = true
```

3.编译
切换到项目的根目录下（就是README.md所在的文件夹），输入：
```
cargo build -r
```
现在项目根目录下会多出一个target文件夹，里面有个release文件夹，再里面有一个名为vnpyrs-chart的可执行文件，这个就是VnpyRS要调用的极速K线图表程序

## 使用

1.和vnpy一样，在家目录下建立一个名为“strategies”的文件夹，在里面新建一个名为“__init__.py”的空文件，再将包含策略的py文件放到“strategies”文件夹里。

2.运行VnpyRS项目里的examples文件夹下的gui.py：
```
python gui.py
```
在第一次点击“K线图表”的时候，需要选择vnpyrs-chart可执行文件的位置，以后使用无需选择。若需要修改，该位置保存在家目录的“.vntrader/vnpyrs.json”中。

## 更新日志

0.1.0：第一个发布版本(2025-3-7)

0.1.1：修复当阳线、阴线、平线中有一种线不存在时，软件会崩溃的问题(2025-3-20)