# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

目前主线在rust版本, 原始java版本只做参考, 不做后续更新

## Project Overview

untar 是一个跨平台的命令行解压工具，支持 tar、gzip、xz、bzip2、zip、7z、rar、lha、lzh、iso、xar、cab、cpio、ar、squashfs、rpm、legacy 格式（ACE/ARC/ZOO/Unix compress）以及单文件压缩流（gz/bz2/xz/zst/lz4/br/lzma/lzo/lz）等。

使用方式为

```shell
untar [参数] 文件名
```

## Architecture

rust

