// 定义处理主题变化的函数
function handleThemeChange(e) {
    if (e.matches) {
        // 用户选择了深色模式
        document.documentElement.setAttribute('data-bs-theme', 'dark')
    } else {
        // 用户选择了浅色模式
        document.documentElement.setAttribute('data-bs-theme', 'light');
    }
}

// 检测当前的深色模式状态
const darkModeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

// 初始化设置
handleThemeChange(darkModeMediaQuery);

// 监听主题切换
darkModeMediaQuery.addEventListener("change", handleThemeChange);
