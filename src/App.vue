<script setup>
import { onMounted, onUnmounted, computed, ref, watchEffect, reactive } from 'vue';
import {
  useOsTheme,
  darkTheme,
  lightTheme,
  NTabs,
  NSwitch,
  NInputNumber,
} from 'naive-ui';
import {
  sendNotification,
  isPermissionGranted,
  requestPermission,
} from '@tauri-apps/plugin-notification';
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import LogPreview from './components/Log.vue';
import { Store } from '@tauri-apps/plugin-store';
import { GlassesOutline, Glasses, DocumentOutline } from '@vicons/ionicons5';
import { event } from '@tauri-apps/api'; 
import { invoke } from '@tauri-apps/api/core';


const tabs = [
  {
    name: '通用',
    code: 'General',
    icon: DocumentOutline,
  },
  {
    name: '日志',
    code: 'Log',
  },
];

const tabActive = ref('General');
const enableAutoStart = ref(false);
const osThemeRef = useOsTheme();
const theme = computed(() =>
  osThemeRef.value === 'dark' ? darkTheme : lightTheme
);

const clientOptions = reactive({
  checkInteral: 1, // 网络检查的时间间隔
  repeatInterval: 3, // 重连模式的间隔
  password: '', // 电脑密码
  connectModel: true, // 启动模式，false为检查模式，true为重连模式
});

const started = ref(false);
const connected = ref(0);
const isFirst = ref(true);
const interval = ref(null);
// 网络状态映射表
const connectedMap = {
  0: '未连接',
  1: '网络正常',
  2: '未联网',
};
const logText = ref([]);

const clientOptionsMix = computed(() => JSON.stringify(clientOptions));

const getLogTime = () => new Date().toLocaleTimeString();

/**
 * 这是一个异步函数，用于发送通知。首先检查是否已经获得了权限，如果没有，则请求权限。如果权限被授予，就发送通知。
 *
 * @param {Object} options - 需要发送的通知的选项，包括通知的内容、标题等。
 * @return {Promise<void>}  无返回值。这个函数是一个异步函数，执行完毕后会发送通知或者结束。
 */
const sendNotify = async (options) => {
  let permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === 'granted';
  }
  if (permissionGranted) {
    sendNotification(options);
  }
};

/**
 * @description 开启或者关闭程序自启动
 */
const changeAutoStart = async () => {
  if (enableAutoStart.value) {
    await disable();
  } else {
    await enable();
  }
  checkIsEnableAutoStart();
};

async function setupSystemEventListeners() {
  try {
    await event.listen('tauri://focus', () => {
      console.log('System is going to sleep.');
    });

    await event.listen('system-resume', () => {
      console.log('System has woken up.');
    });
  } catch (error) {
    console.error('Error setting up system event listeners:', error);
  }
}

const checkNetworkStatus = () => {
  try {
    invoke('get_network_status').then((res) => {
      if (['Wifi', 'Ethernet'].includes(res)) {
        // network connected, check status
        invoke('check_network').then((res) => {
          connected.value = res ? 1 : 2;
        });
      } else {
        connected.value = 0;
      }
    });
  } catch (e) {
    logText.value.push({
      text: String(e),
      type: 'warning',
      time: getLogTime(),
    });
  }
};

/**
 * 自动检查网络状态的函数。
 * 该函数首先立即执行一次网络状态检查，然后根据设定的时间间隔周期性地执行网络状态检查。
 * 使用异步方式进行网络状态检查，并在首次调用时将 `isFirst` 标志设置为 `false`。
 * 定时器在组件销毁时需要被清除。
 */
const autoCheckNetworkStatus = async () => {
  // 定时检查，销毁时清除定时器, 默认进来检查一次
  checkNetworkStatus();
  if (isFirst.value) isFirst.value = false;
  interval.value = setInterval(async () => {
    if (isFirst.value) isFirst.value = false;
    checkNetworkStatus();
  }, clientOptions.checkInteral * 1000);
};

const checkIsEnableAutoStart = async () => {
  enableAutoStart.value  = await isEnabled();
};

/**
 * 用于从缓存中获取密码。如果缓存中有密码，则将其赋值给password.value。
 *
 * @async
 * @return {Promise<void>} 无返回值，但会改变全局变量password.value的值。
 */
const getClientOptionsFromCache = async () => {
  const store = await Store.load('settings.json');
  const optionsCacheStr = await store.get('options');
  if (optionsCacheStr) {
    const optionsCache = JSON.parse(optionsCacheStr);
    if (optionsCache.password) clientOptions.password = optionsCache.password;
    if (optionsCache.checkInteral) clientOptions.checkInteral = optionsCache.checkInteral;
    if (optionsCache.connectModel) clientOptions.connectModel = optionsCache.connectModel;
    if (optionsCache.repeatInterval) clientOptions.repeatInterval = optionsCache.repeatInterval;
  }
};

/**
 * 用于将密码存储到缓存中。如果密码值为空，则直接返回；否则，将密码值存储到名为'password'的缓存中。
 *
 * @async
 * @function setPasswordToCache
 * @return {Promise<void>} 无返回值，但会改变缓存中的'password'项的值。
 */
const setClientOptionCache = async () => {
  if (!clientOptions.password) return;
  const store = await Store.load('settings.json');
  await store.set('options', JSON.stringify(clientOptions));
};

function debounce(func, wait) {
  let timeout;

  return function(...args) {
    const context = this;

    clearTimeout(timeout);
    timeout = setTimeout(() => {
      func.apply(context, args);
    }, wait);
  };
}



watchEffect(() => {
  if (clientOptionsMix.value) {
    // 防抖处理，避免频繁写入缓存
    debounce(setClientOptionCache, 500)();
  }
});

watchEffect(() => {
  if (connected.value === 2 && !isFirst.value) {
    logText.value.push({
      text: '网络异常开始重新连接',
      type: 'info',
      time: getLogTime(),
    });
    sendNotify({
        title: 'Sauter',
        body: '网络异常开始重新连接',
    });
    runScript();
  }
});

/**
 * @description 启停服务函数。如果服务已经启动，则停止服务并清除定时器；否则，启动服务并设置定时器。
 */
const startOrStpService = () => {
  if (started.value) {
    started.value = false;
    clearInterval(interval.value);
  } else {
    if (!clientOptions.password) {
      // 如果没有密码的情况，服务不能启动
      sendNotify({
        title: 'Sauter',
        body: '请先设置密码',
      });
      logText.value.push({
        text: '请先设置密码',
        type: 'warning',
        time: getLogTime(),
      });
      return;
    }
    if (connected.value === 0) {
      sendNotify({
        title: 'Sauter',
        body: '请先连接 wifi 或者以太网',
      });
      logText.value.push({
        text: '请先连接 wifi 或者以太网',
        type: 'warning',
        time: getLogTime(),
      });
      return;
    }
    startService().then(() => {
      started.value = true;
      sendNotify({
        title: 'Sauter',
        body: '服务已启动',
      });
    });
  }
};


const startService = () => {
  return new Promise((resolve, reject) => {
    try {
      logText.value.push({
          text: `服务将以${clientOptions.connectModel ? '【重连模式】' : '【检查模式】'}启动`,
          type: 'info',
          time: getLogTime(),
      });
      if (clientOptions.connectModel) {
        // 定时运行重连
        // 1. 清除定时任务
        clearInterval(interval.value);
        // 2. 设置定时任务, 执行重连方法
        runScript();
        interval.value = setInterval(async () => {
          runScript();
        }, clientOptions.repeatInterval * 60000);
        resolve();
      } else {
        autoCheckNetworkStatus();
        resolve();
      }
    } catch {
      reject();
    }
  });
};

async function runScript() {
  if (!clientOptions.password) {
    sendNotify({
      title: 'Sauter',
      body: '请先设置密码',
    });
  } else {
    try {
      const result = await invoke('stop_inode_services');
      if (result === '1') {
        const startSuccessCode = await invoke('start_inode_services', {
          password: clientOptions.password,
        });
        logText.value.push({
          text: startSuccessCode,
          type: 'success',
          time: getLogTime(),
        });
      } else {
        logText.value.push({
          text: result,
          type: 'error',
          time: getLogTime(),
        });
      }
    } catch (error) {
      logText.value.push({
        text: error,
        type: 'error',
        time: getLogTime(),
      });
    }
  }
}

onMounted(async () => {
  setupSystemEventListeners();
  // get password from cache
  await getClientOptionsFromCache();
  // initial and check network status
  autoCheckNetworkStatus();
  // check auto start status
  checkIsEnableAutoStart();
});

onUnmounted(() => {
  clearInterval(interval.value);
});
</script>

<template>
  <n-config-provider :theme="theme">
    <header class="px-4 h-12">
      <n-tabs ref="tabsInstRef" v-model:value="tabActive" size="large">
        <n-tab v-for="tab in tabs" :key="tab.code" :name="tab.code">
          <div class="flex font-bold">
            {{ tab.name }}
          </div>
        </n-tab>
      </n-tabs>
    </header>
    <div class="container flex flex-col">
      <main class="main flex-1 flex flex-col px-4">
        <!-- General Setting -->
        <div
          v-if="tabActive === 'General'"
          class="flex-col py-1 w-full"
          :class="{ general: tabActive === 'General' }"
        >
          <!-- network status -->
          <div class="flex items-center border py-2">
            <label for="status" class="mr-2 font-bold w-20 label leading-8"
              >网络状态:</label
            >
            <div
              clas="font-bold weight-bold flex items-center"
              :class="[
                'text-base',
                connected ? 'text-green-500' : 'text-red-500',
              ]"
            >
              <Icon
                class="animate-icon"
                :name="connected ? 'connected' : 'disconnect'"
              />
              <span class="ml-2"> {{ connectedMap[connected] }}</span>
            </div>
          </div>
          <div class="flex items-center border py-2">
            <label for="password" class="mr-2 font-bold w-20 label leading-8"
              >电脑密码:
            </label>
            <div class="flex items-center flex-1">
              <!-- 输入密码 -->
              <n-input
                type="password"
                :show-password-on="'mousedown'"
                v-model:value="clientOptions.password"
                placeholder="请输入电脑密码"
                style="width: 14rem"
                @on-input="setClientOptionCache"
                @keyup.enter="setClientOptionCache"
              >
                <template #password-visible-icon>
                  <n-icon :size="16" :component="GlassesOutline" />
                </template>
                <template #password-invisible-icon>
                  <n-icon :size="16" :component="Glasses" />
                </template>
              </n-input>
            </div>
          </div>
          <div class="flex items-center border py-2">
            <label for="launch" class="mr-2 font-bold w-20 label leading-8"
              >轮询间隔：</label
            >
            <div class="flex items-center flex-1">
              <n-input-number
                :show-button="false"
                style="width: 14rem"
                placeholder="轮询网络检查时间间隔"
                v-model:value="clientOptions.checkInteral"
                :min="1"
              >
              <template #suffix>秒</template>
            </n-input-number>
            </div>
          </div>
          <div class="flex items-center border py-2" v-if="clientOptions.connectModel">
            <label for="launch" class="mr-2 font-bold w-20 label leading-8"
              >重连间隔：</label
            >
            <div class="flex items-center flex-1">
              <n-input-number
                :show-button="false"
                style="width: 14rem"
                placeholder="重连时间间隔"
                v-model:value="clientOptions.repeatInterval"
                :min="1"
              >
              <template #suffix>分钟</template>
            </n-input-number>
            </div>
          </div>
          <div class="flex items-center border py-2">
            <label for="launch" class="mr-2 font-bold w-20 label leading-8"
              >重连模式：</label
            >
            <div class="flex items-center cursor-pointer pl-1">
              <n-switch :round="false" v-model:value="clientOptions.connectModel">
                <template #checked>
                  重连模式
                </template>
                <template #unchecked>
                  检查模式
                </template>
              </n-switch>
            </div>
          </div>
          <div class="flex items-center border py-2">
            <label for="launch" class="mr-2 font-bold w-20 label leading-8"
              >启动项：</label
            >
            <div class="flex items-center cursor-pointer pl-1" :class="[enableAutoStart ? 'text-green-500' : 'text-gray-500']" @click="changeAutoStart">
              {{ enableAutoStart ? '注销' : '注册' }}
            </div>
          </div>
        </div>
        <!-- Start Button -->
        <div
          class="btn w-full flex justify-end pb-2"
          v-if="tabActive === 'General'"
        >
          <button
            class="custom-btn px-2 py-1 rounded-md flex justify-center items-center"
            @click="startOrStpService"
          >
          <span>{{ started ? "停止" : "启动" }}</span>
          <Icon :name="!started ? 'start' : 'pause'" :size="15"  :class="['ml-2', started ? 'text-yellow-600': 'text-green-500', !started ? '' : 'starting']"/>
          </button>
        </div>
        <LogPreview v-if="tabActive === 'Log'" v-model:text="logText" />
      </main>
    </div>
  </n-config-provider>
</template>

<style lang="scss" scoped>
.n-config-provider {
  width: 100%;
  height: 100%;
}

header {
  background-color: var(--page-head-background-color);
}

.container {
  width: 100vw;
  height: calc(100vh - 3rem);
  box-sizing: border-box;
  .main {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    .general {
      height: calc(100% - 2rem);
    }
  }
  @keyframes circle-in-circle {
    0% {
        transform-origin: center center;
        transform: translate(0,0) rotate(0deg);
    }
    50% {
        transform-origin: 50% 50%;
        transform: translate(1px,0) rotate(180deg);
    }
    100% {
        transform-origin: center center;
        transform: translate(0,0) rotate(360deg);
    }
    
  }
  .starting {
    animation: 4s circle-in-circle infinite;
  }
}
</style>
