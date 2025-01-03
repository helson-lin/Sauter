import { createApp } from 'vue';
import App from './App.vue';
// 通用字体
import { InstallCodeMirror } from 'codemirror-editor-vue3';
import 'vfonts/Lato.css';
// 等宽字体
import 'vfonts/FiraCode.css';
import './assets/css/tailwind.css';
import './assets/css/global.css';
import './assets/css/resetQuill.css';

import 'virtual:svg-icons-register';
import Icon from './components/Icon.vue';

import {
  // create naive ui
  create,
  // component
  NInput,
  NIcon,
  NSplit,
  NButton,
  NDropdown,
  NPopover,
  NConfigProvider,
  NTab,
  NTabs
} from 'naive-ui';

const naive = create({
  components: [NIcon, NButton, NInput, NSplit, NPopover, NDropdown, NConfigProvider, NTabs, NTab],
});
const app = createApp(App);
app.component('Icon', Icon);
app.use(naive);
app.use(InstallCodeMirror);
app.mount('#app');
