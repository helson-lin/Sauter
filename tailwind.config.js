/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './src/**/*.{html,js,vue}'
  ],
  theme: {
    extend: {
      colors: {
        mpink: '#966FE0', // 自定义颜色名和对应的颜色值
        primary: {
          light: '#C39FD9', // 自定义颜色的不同色调
          DEFAULT: '#966FE0',
          dark: '#7A33B6',
        },
      },
      extend: {
        height: {
          '19': '4.5rem'
        },
      },
    },
  },
  plugins: [],
};

