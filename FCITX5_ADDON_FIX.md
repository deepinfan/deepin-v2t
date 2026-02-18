# Fcitx5 输入法识别问题修复

## 问题现象

安装 DEB 包后，在 Fcitx5 配置中找不到"水滴语音输入法"。

日志显示：
```
I2026-02-18 09:40:42.995508 addonmanager.cpp:205] Loaded addon vinput-addon
I2026-02-18 09:40:43.058283 inputmethodmanager.cpp:209] Found 0 input method(s) in addon vinput-addon
```

**Addon 加载成功，但没有找到任何输入法！**

## 根本原因

Fcitx5 使用**配置文件名**（去掉 .conf）作为 addon 的内部名称：

- `vinput-addon.conf` → addon 名称是 `vinput-addon`
- `pinyin.conf` → addon 名称是 `pinyin`

但是 `vinput.conf` 中错误地写成：
```ini
Addon=vinput  # ❌ 错误：不存在名为 vinput 的 addon
```

应该是：
```ini
Addon=vinput-addon  # ✅ 正确：匹配 vinput-addon.conf
```

## 解决方案

修改 `fcitx5-vinput/vinput.conf`：

```diff
[InputMethod]
Name=水滴语音输入法
Icon=audio-input-microphone
Label=语
LangCode=zh_CN
-Addon=vinput
+Addon=vinput-addon
Configurable=False
```

## 验证

修复后，日志应该显示：
```
I2026-02-18 XX:XX:XX addonmanager.cpp:205] Loaded addon vinput-addon
I2026-02-18 XX:XX:XX inputmethodmanager.cpp:209] Found 1 input method(s) in addon vinput-addon
```

## 参考

其他输入法的配置示例：

**pinyin.conf**:
```ini
[InputMethod]
Name=Pinyin
Addon=pinyin  # 匹配 pinyin.conf
```

**pinyin.conf (addon)**:
```ini
[Addon]
Name=Pinyin
Library=libpinyin
```

## 已修复

- ✅ 修改了 vinput.conf
- ✅ 重新打包 DEB
- ✅ 提交到 GitHub

新的 DEB 包：`droplet-voice-input_0.1.0_amd64.deb` (2026-02-18 09:45)
