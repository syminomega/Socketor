export function RemoveLoadingIndicator() {
    // 所有资源加载完成后，淡出并移除加载组件
    const loadingContainer = document.querySelector('.loading-container');
    if (loadingContainer) {
        loadingContainer.style.transition = 'opacity 0.5s ease';
        loadingContainer.style.opacity = '0';
        setTimeout(() => {
            if (loadingContainer) {
                loadingContainer.style.display = 'none';
            }
        }, 500);
    }
}
