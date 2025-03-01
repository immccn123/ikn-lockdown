# 快速开始

前往 <https://github.com/immccn123/ikn-lockdown/releases> 下载最新版本。

当前版本：<a target="_blank" id="download"><span id="version">正在加载……</span></a>

<script>
    fetch("https://api.github.com/repos/immccn123/ikn-lockdown/releases")
      .then(x => x.json())
      .then(x => x[0])
      .then(x => {
        document.getElementById("version").innerText = x.name
        document.getElementById("download").href = x.html_url
      })
      .catch(x => document.getElementById("version").innerText = `获取版本时出现错误：${x}`)
</script>

下载解压后启动 lockdown-panel.exe 即可使用。点击「启动服务」按钮可以启动 Lockdown 的背景服务。
