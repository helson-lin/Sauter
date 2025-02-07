<template>
  <div class="log w-full h-full overflow-y-auto" ref="logRef" id="logDom">
    <p
      v-for="item in htmlString"
      :key="item.text"
      :class="[
        `log-${item.type}`,
        'log-item leading-8 border-spacing-1 border border-b-gray-400',
      ]"
    >
      <span :class="['font-bolder rounded-md', getBackGroundColor(item.type)]"
        >{{ item.time }}
      </span>
      {{ item.text }}
    </p>
  </div>
</template>
<script>
import { defineComponent, computed, ref } from 'vue';
export default defineComponent({
  props: {
    text: {
      type: Array,
      required: true,
    },
  },
  setup(props) {
    const logRef = ref(null);
    const htmlString = computed(() => {
      // 创建 props.text 的副本并反转
      return props.text ? [...props.text].reverse() : null;
    });
    const getBackGroundColor = (type) => {
      if (type === 'success') {
        return 'back-success text-black px-2';
      }
      if (type === 'warning') {
        return 'back-warning text-black px-2';
      }
      if (type === 'error') {
        return 'back-error text-black px-2';
      }
      return ' bg-gray-200 px-2';
    };
    return {
      htmlString,
      getBackGroundColor,
      logRef,
    };
  },
});
</script>

<style lang="scss" scoped>
#logDom {
  color: var(--primary-color);
}

.log-success {
  color: #18a058;
}

.log-error {
  color: rgba(192, 26, 22, 1);
}

.log-warning {
  color: #eb4a3a;
}

.back-success {
  background-color: #18a058;
}

.back-warning {
  background-color: #eb4a3a;
}

.back-error {
  background-color: rgba(192, 26, 22, 1);
}

.log-title {
  font-weight: bold;
}

.log-item {
  font-weight: 500;
}
</style>
