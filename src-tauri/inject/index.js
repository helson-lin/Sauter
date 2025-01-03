console.warn('injected');

// 拦截 window.open
const originalWindowOpen = window.open;
window.open = function (url, target, features) {
    return originalWindowOpen.call(window, url, '_self', features);
};
// a 链接代理
// const originalAElementClick = document.a.onclick;
document.addEventListener('click', function (event) {
    const target = event.target.closest('a'); // 查找最近的 <a> 标签
    console.warn(target);
    if (target) {
        event.preventDefault(); // 阻止默认行为
        const url = target.href; // 获取链接的 URL

        // 在这里可以执行自定义逻辑，例如使用 Tauri API 打开链接
        console.log(`Intercepted link: ${url}`);

        window.open(url, '_self');
    }
});