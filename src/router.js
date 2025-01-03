import { createRouter, createWebHashHistory } from 'vue-router';


const routes = [
    {
        path: '/',
        name: 'index',
        meta: {
            title: '首页',
            keepAlive: true,
            requireAuth: true,
        },
        component: () => import('./pages/Index.vue'),
    }
];

const router = createRouter({
    history: createWebHashHistory(),
    routes,
});

router.beforeEach((to, from, next) => {
    const { title } = to.meta;
    if (title) {
        document.title = `${title}`;
    }
    next();
});
export default router;
