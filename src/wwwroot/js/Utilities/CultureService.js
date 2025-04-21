export function getStoredCulture() {
    const storedCulture = localStorage.getItem('culture');
    if (storedCulture) {
        document.documentElement.lang = storedCulture;
        return storedCulture;
    }
    // 如果没有存储的文化信息，返回浏览器默认语言，使用en-US作为默认值
    const browserLanguage = navigator.language;
    document.documentElement.lang = 'en-US';
    return browserLanguage;
}

export function setStoredCulture(culture) {
    localStorage.setItem('culture', culture);
    document.documentElement.lang = culture;
}