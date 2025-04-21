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

// 平滑滚动到div底部
// function scrollToBottom(element) {
//     element.scrollTop = element.scrollHeight;
// }
function scrollToBottom(element) {
    const start = element.scrollTop;
    const end = element.scrollHeight;
    const distance = end - start;
    const duration = 500; // 动画持续时间（毫秒）
    const startTime = performance.now();

    function scrollStep(currentTime) {
        const elapsed = currentTime - startTime;
        const progress = Math.min(elapsed / duration, 1); // 确保进度不超过1
        element.scrollTop = start + distance * progress;

        if (progress < 1) {
            window.requestAnimationFrame(scrollStep);
        }
    }

    window.requestAnimationFrame(scrollStep);
}